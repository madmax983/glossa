//! Control flow analysis for ΓΛΩΣΣΑ
//!
//! This module handles the analysis of control flow statements:
//! - Conditionals (εἰ, ἐάν, εἰ δὲ μή)
//! - Loops (ἕως, ἀπὸ ... μέχρι, διὰ ...)
//! - Pattern matching (κατά)
//! - Return (δός)
//! - Break/Continue (παῦε, συνέχιζε)

use super::conversion::convert_assembled_to_analyzed;
use super::expressions::get_first_word;
use super::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope,
    analyzer::analyze_statement, assemble_statement,
};
use crate::ast::{Clause, Expr, Statement};
use crate::errors::GlossaError;
use crate::limits::MAX_CONTROL_FLOW_DEPTH;
use crate::morphology::lexicon;

/// Intercepts and parses control flow constructs (`εἰ`, `ἕως`, `διὰ`) before standard assembly.
///
/// In GLOSSA, control flow statements span multiple clauses (e.g., condition, body, else body)
/// and do not map cleanly to the simple Subject-Object-Verb triples expected by the
/// [`crate::semantic::Assembler`]. This function acts as a pre-processor: if a statement begins
/// with a known control flow particle, it bypasses standard assembly entirely and is parsed
/// directly into an [`AnalyzedStatement`] using structural heuristics.
///
/// # Returns
///
/// * `Ok(Some(AnalyzedStatement))` if it was successfully parsed as control flow.
/// * `Ok(None)` if the statement does not start with a control flow particle (should be assembled normally).
/// * `Err(GlossaError)` if it is malformed control flow (e.g., missing condition).
///
/// # Examples
///
/// ```rust,ignore
/// // Example cannot be run as a doctest because this module is pub(crate)
/// use glossa::ast::{Statement, Clause, Expr, Word};
/// use glossa::semantic::resolver::Scope;
/// use glossa::semantic::control_flow::analyze_control_flow;
///
/// // Represents: "ξ 5 ἔστω" (Not control flow)
/// let mut scope = Scope::new();
/// let stmt = Statement::Regular {
///     clauses: vec![Clause {
///         expressions: vec![Expr::Word(Word::new("ξ"))],
///     }],
///     is_query: false,
///     is_propagate: false,
/// };
///
/// let result = analyze_control_flow(&stmt, &mut scope).unwrap();
/// assert!(result.is_none());
/// ```
pub fn analyze_control_flow(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    // Note: Function definitions are handled in `mod.rs` before calling this.

    // Get the first word to check for control flow particles
    // If this fails (e.g., statement starts with literal), it's not control flow
    let normalized = match get_first_word(stmt) {
        Ok(word) => word,
        Err(_) => return Ok(None),
    };

    // Conditional: εἰ/ἐάν condition, body [, εἰ δὲ μή, else_body]
    if lexicon::is_conditional_particle(&normalized) {
        return parse_conditional(stmt, scope, 0);
    }

    // While loop: ἕως condition, body
    if normalized == "εως" {
        return parse_while_loop(stmt, scope);
    }

    // For loop with range: ἀπὸ start μέχρι/ἕως end, body
    if normalized == "απο" {
        return parse_for_range_loop(stmt, scope);
    }

    // For loop with iteration: διὰ collection, body
    if normalized == "δια" {
        return parse_for_iteration_loop(stmt, scope);
    }

    if lexicon::is_loop_particle(&normalized) {
        // Other loop forms handled above
        return Ok(None);
    }

    if lexicon::is_match_particle(&normalized) {
        return parse_match_expression(stmt, scope);
    }

    // Return: δός (give)
    if normalized == "δος" {
        return parse_return_statement(stmt, scope);
    }

    // Break: παῦε
    if lexicon::is_break_verb(&normalized) {
        return Ok(Some(AnalyzedStatement::Break));
    }

    // Continue: συνέχιζε
    if lexicon::is_continue_verb(&normalized) {
        return Ok(Some(AnalyzedStatement::Continue));
    }

    // Not a control flow construct
    Ok(None)
}

/// Parse a while loop statement (ἕως)
/// Structure: ἕως condition, body
fn parse_while_loop(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if stmt.clauses().len() < 2 {
        return Err(GlossaError::semantic(
            "While loop needs at least 2 clauses: condition and body",
        ));
    }

    // Parse condition (first clause, minus the ἕως particle)
    let condition_clause = &stmt.clauses()[0];
    let condition_expr = skip_first_word_and_parse(condition_clause, scope)?;

    // Parse body - all remaining clauses
    let body_clauses = &stmt.clauses()[1..];

    let body_stmt = Statement::Regular {
        clauses: body_clauses.to_vec(),
        is_query: false,
        is_propagate: false,
    };

    // Analyze body using unified helper
    let body_analyzed = analyze_statement(&body_stmt, scope)?;

    Ok(Some(AnalyzedStatement::While {
        condition: Box::new(condition_expr),
        body: body_analyzed,
    }))
}

fn parse_range_bound(
    expr: &Expr,
    scope: &Scope,
    bound_type: &str,
) -> Result<AnalyzedExpr, GlossaError> {
    if let Expr::Word(w) = expr {
        // Parse as number literal
        if let Some(val) = lexicon::numeral_value(&w.normalized) {
            Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(val),
                glossa_type: GlossaType::Number,
            })
        } else {
            // Try as variable
            Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(w.normalized.clone()),
                glossa_type: scope
                    .lookup(&w.normalized)
                    .cloned()
                    .unwrap_or(GlossaType::Number),
            })
        }
    } else {
        Err(GlossaError::semantic(format!(
            "Expected word for range {}",
            bound_type
        )))
    }
}

/// Parse a for loop with range (ἀπὸ ... μέχρι/ἕως ...)
/// Structure: ἀπὸ start μέχρι/ἕως end, body
fn parse_for_range_loop(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if stmt.clauses().len() < 2 {
        return Err(GlossaError::semantic(
            "For loop needs at least 2 clauses: range and body",
        ));
    }

    // Parse range specification from first clause
    let range_clause = &stmt.clauses()[0];

    if range_clause.expressions.is_empty() {
        return Err(GlossaError::semantic("Empty range clause in for loop"));
    }

    // Extract words from the first expression (should be a Phrase)
    let words = if let Expr::Phrase(terms) = &range_clause.expressions[0] {
        terms
    } else {
        return Err(GlossaError::semantic("Expected phrase in for range"));
    };

    // Pattern: ἀπὸ start μέχρι/ἕως end
    // We need at least 4 words
    if words.len() < 4 {
        return Err(GlossaError::semantic(
            "For range needs: ἀπὸ start μέχρι/ἕως end",
        ));
    }

    // Extract start (word at index 1)
    let start_expr = parse_range_bound(&words[1], scope, "start")?;

    // Check if it's inclusive (ἕως) or exclusive (μέχρι) - word at index 2
    let inclusive = if let Expr::Word(w) = &words[2] {
        w.normalized == "εως"
    } else {
        false
    };

    // Extract end (word at index 3)
    let end_expr = parse_range_bound(&words[3], scope, "end")?;

    let iterator = AnalyzedExpr {
        expr: AnalyzedExprKind::Range {
            start: Box::new(start_expr),
            end: Box::new(end_expr),
            inclusive,
        },
        glossa_type: GlossaType::Number,
    };

    // Parse body - all remaining clauses form the body
    let body_clauses = &stmt.clauses()[1..];

    if body_clauses.is_empty() {
        return Err(GlossaError::semantic("For loop needs a body"));
    }

    // Extract variable name from first word of first body clause
    let variable = if let Some(first_expr) = body_clauses[0].expressions.first() {
        if let Expr::Phrase(terms) = first_expr {
            if let Some(Expr::Word(w)) = terms.first() {
                w.normalized.clone()
            } else {
                "i".into()
            }
        } else if let Expr::Word(w) = first_expr {
            w.normalized.clone()
        } else {
            "i".into()
        }
    } else {
        "i".into()
    };

    let body_stmt = Statement::Regular {
        clauses: body_clauses.to_vec(),
        is_query: false,
        is_propagate: false,
    };

    // Use a scope guard to ensure the loop variable is removed after parsing
    let body_analyzed = {
        let mut loop_scope = scope.enter_scope();
        loop_scope.define(variable.clone(), GlossaType::Number);

        analyze_statement(&body_stmt, &mut loop_scope)?
    };

    Ok(Some(AnalyzedStatement::For {
        variable,
        iterator: Box::new(iterator),
        body: body_analyzed,
    }))
}

/// Parse a for loop with iteration (διὰ collection)
/// Structure: διὰ collection_genitive, body
/// Example: διὰ στοιχείων, στοιχεῖον λέγε (through elements, say element)
fn parse_for_iteration_loop(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if stmt.clauses().len() < 2 {
        return Err(GlossaError::semantic(
            "For loop needs at least 2 clauses: collection and body",
        ));
    }

    // Extract collection name from first clause (διὰ + genitive noun)
    let collection_clause = &stmt.clauses()[0];

    if collection_clause.expressions.is_empty() {
        return Err(GlossaError::semantic("Empty collection clause in for loop"));
    }

    let (collection_name_raw, collection_name) =
        if let Expr::Phrase(terms) = &collection_clause.expressions[0] {
            // Skip διά (first word) and get the collection name (second word)
            if terms.len() < 2 {
                return Err(GlossaError::semantic("For iteration needs: διὰ collection"));
            }
            if let Expr::Word(w) = &terms[1] {
                // Get lemma of the word to match the definition
                let lemma = crate::morphology::analyze(&w.normalized).lemma.to_string();
                (w.normalized.clone(), lemma)
            } else {
                return Err(GlossaError::semantic("Expected word for collection"));
            }
        } else {
            return Err(GlossaError::semantic("Expected phrase in for iteration"));
        };

    // Create a variable expression for the collection
    let collection_type = scope
        .lookup(&collection_name)
        .cloned()
        .ok_or_else(|| GlossaError::undefined(collection_name_raw.to_string()))?;
    let collection_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::Variable(collection_name.into()),
        glossa_type: collection_type,
    };

    // Parse body - all remaining clauses
    let body_clauses = &stmt.clauses()[1..];

    // Extract variable name from first word of body
    let variable = if let Some(first_expr) = body_clauses[0].expressions.first() {
        if let Expr::Phrase(terms) = first_expr {
            if let Some(Expr::Word(w)) = terms.first() {
                w.normalized.clone()
            } else {
                "x".into()
            }
        } else if let Expr::Word(w) = first_expr {
            w.normalized.clone()
        } else {
            "x".into()
        }
    } else {
        "x".into()
    };

    let body_stmt = Statement::Regular {
        clauses: body_clauses.to_vec(),
        is_query: false,
        is_propagate: false,
    };

    // Use scope guard for loop variable
    let body_analyzed = {
        let mut loop_scope = scope.enter_scope();
        // TODO: Infer element type from collection type
        loop_scope.define(variable.clone(), GlossaType::String);

        analyze_statement(&body_stmt, &mut loop_scope)?
    };

    Ok(Some(AnalyzedStatement::For {
        variable,
        iterator: Box::new(collection_expr),
        body: body_analyzed,
    }))
}

/// Parse a match expression (κατά)
/// Structure: κατὰ scrutinee· pattern₁ ᾖ, body₁· pattern₂ ᾖ, body₂· pattern₃ ᾖ, body₃.
/// Example: κατὰ ξ· μηδὲν ᾖ, «μηδέν» λέγε· ἓν ᾖ, «ἕν» λέγε· ἄλλο ᾖ, «ἄλλο» λέγε.
fn parse_match_expression(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if stmt.clauses().is_empty() {
        return Err(GlossaError::semantic(
            "Match expression needs at least one clause",
        ));
    }

    // Extract scrutinee from clause 0, expression 0 (skip κατά)
    let scrutinee_clause = &stmt.clauses()[0];
    // Create a synthetic clause with only the first expression to avoid including the first pattern (expr 1)
    let synthetic_clause = Clause {
        expressions: vec![scrutinee_clause.expressions[0].clone()],
    };
    let scrutinee = skip_first_word_and_parse(&synthetic_clause, scope)?;

    // Build arms: pattern from clause[i], expr 1 → body from clause[i+1], expr 0
    let mut arms = Vec::with_capacity(stmt.clauses().len() / 2);

    for i in 0..stmt.clauses().len() {
        let clause = &stmt.clauses()[i];

        // Get pattern from expression 1 (if it exists)
        if clause.expressions.len() > 1 {
            let pattern_expr = &clause.expressions[1];

            // Extract pattern value
            let pattern = parse_match_pattern(pattern_expr, scope)?;

            // Get body from next clause, expression 0
            if i + 1 >= stmt.clauses().len() {
                return Err(GlossaError::semantic("Match pattern without body"));
            }

            let body_clause = &stmt.clauses()[i + 1];
            if body_clause.expressions.is_empty() {
                return Err(GlossaError::semantic("Empty match arm body"));
            }

            // Parse body as a statement
            let body_expr_clause = Clause {
                expressions: vec![body_clause.expressions[0].clone()],
            };
            let body_stmt = Statement::Regular {
                clauses: vec![body_expr_clause],
                is_query: false,
                is_propagate: false,
            };
            let body_analyzed = analyze_statement(&body_stmt, scope)?;

            arms.push((pattern, body_analyzed));
        }
    }

    if arms.is_empty() {
        return Err(GlossaError::semantic(
            "Match expression needs at least one arm",
        ));
    }

    Ok(Some(AnalyzedStatement::Match {
        scrutinee: Box::new(scrutinee),
        arms,
    }))
}

/// Parse a return statement: δός value
fn parse_return_statement(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    // Get the first clause
    if stmt.clauses().is_empty() {
        return Ok(Some(AnalyzedStatement::Return { value: None }));
    }

    let clause = &stmt.clauses()[0];

    // Try to parse return expression
    let return_expr = parse_return_expression(clause, scope)?;

    Ok(Some(AnalyzedStatement::Return {
        value: Some(Box::new(return_expr)),
    }))
}

/// Parse return expression in a simple way
fn parse_return_expression(clause: &Clause, scope: &Scope) -> Result<AnalyzedExpr, GlossaError> {
    // For Cycle 3, we'll do simple expression parsing
    // The expression after δός could be:
    // - A simple variable: ξ
    // - A literal: πέντε
    // - An operation: ξ δύο ἄθροισμα

    // ⚡ Bolt Optimization: Uses a slice of the terms array instead of collecting into a new Vec.
    // This avoids unnecessary O(n) heap allocation during control flow analysis.
    let words = if let Some(Expr::Phrase(terms)) = clause.expressions.first() {
        if terms.len() > 1 { &terms[1..] } else { &[] }
    } else {
        &[]
    };

    if words.is_empty() {
        return Ok(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(0),
            glossa_type: GlossaType::Number,
        });
    }

    // Simple heuristic: if single word or literal
    if words.len() == 1 {
        match &words[0] {
            Expr::Word(w) => {
                let normalized = &w.normalized;

                // Check if it's a numeral
                if let Some(val) = lexicon::numeral_value(normalized) {
                    return Ok(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(val),
                        glossa_type: GlossaType::Number,
                    });
                }

                // Treat as variable
                let var_type = scope
                    .lookup(normalized)
                    .cloned()
                    .unwrap_or(GlossaType::Unknown);
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(normalized.clone()),
                    glossa_type: var_type,
                });
            }
            Expr::NumberLiteral(n) => {
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(*n),
                    glossa_type: GlossaType::Number,
                });
            }
            Expr::StringLiteral(s) => {
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::StringLiteral(s.clone()),
                    glossa_type: GlossaType::String,
                });
            }
            Expr::BooleanLiteral(b) => {
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(*b),
                    glossa_type: GlossaType::Boolean,
                });
            }
            _ => {}
        }
    }

    // For now, just return a number literal placeholder for complex expressions
    // TODO: Implement full expression parsing that respects function scope
    Ok(AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(0),
        glossa_type: GlossaType::Number,
    })
}

/// Parse a match pattern expression
fn parse_match_pattern(expr: &Expr, scope: &mut Scope) -> Result<AnalyzedExpr, GlossaError> {
    // Pattern is typically: value ᾖ
    if let Expr::Phrase(terms) = expr {
        if terms.is_empty() {
            return Err(GlossaError::semantic("Empty match pattern"));
        }

        // Get first word (the pattern value)
        if let Expr::Word(w) = &terms[0] {
            let normalized = &w.normalized;

            // Check if it's ἄλλο (wildcard)
            if normalized == "αλλο" {
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                });
            }

            // Check if it's a numeral
            if let Some(val) = lexicon::numeral_value(normalized) {
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(val),
                    glossa_type: GlossaType::Number,
                });
            }

            // Otherwise, treat as variable reference
            let var_type = scope
                .lookup(normalized)
                .cloned()
                .unwrap_or(GlossaType::Number);
            return Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(normalized.clone()),
                glossa_type: var_type,
            });
        }
    } else if let Expr::Word(w) = expr {
        let normalized = &w.normalized;

        // Check for wildcard
        if normalized == "αλλο" {
            return Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: GlossaType::Boolean,
            });
        }

        // Check for numeral
        if let Some(val) = lexicon::numeral_value(normalized) {
            return Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(val),
                glossa_type: GlossaType::Number,
            });
        }
    }

    Err(GlossaError::semantic("Invalid match pattern"))
}

/// Parse a conditional statement (εἰ/ἐάν)
fn parse_conditional(
    stmt: &Statement,
    scope: &mut Scope,
    depth: usize,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if depth > MAX_CONTROL_FLOW_DEPTH {
        return Err(GlossaError::AssemblyError(crate::errors::AssemblyError::LimitExceeded {
            resource: "Control flow depth".into(),
            max: MAX_CONTROL_FLOW_DEPTH,
        }));
    }

    if stmt.clauses().len() < 2 {
        return Err(GlossaError::semantic(
            "Conditional needs at least 2 clauses: condition and body",
        ));
    }

    // Parse condition (first clause, minus the εἰ/ἐάν particle)
    let condition_clause = &stmt.clauses()[0];
    let condition_expr = skip_first_word_and_parse(condition_clause, scope)?;

    // Parse then-body (second clause, first expression)
    let then_clause = &stmt.clauses()[1];
    let then_body = if then_clause.expressions.is_empty() {
        return Err(GlossaError::semantic("Empty then-body in conditional"));
    } else {
        // Create a mini-clause with just the first expression
        let first_expr_clause = Clause {
            expressions: vec![then_clause.expressions[0].clone()],
        };
        let stmt = Statement::Regular {
            clauses: vec![first_expr_clause],
            is_query: false,
            is_propagate: false,
        };
        analyze_statement(&stmt, scope)?
    };

    // Check for else/elif clause
    let else_body = if then_clause.expressions.len() > 1 && stmt.clauses().len() >= 3 {
        let second_expr = &then_clause.expressions[1];

        // Check if it's "εἰ δὲ μή" (else)
        if check_else_pattern_in_expression(second_expr) {
            let stmt = Statement::Regular {
                clauses: vec![stmt.clauses()[2].clone()],
                is_query: false,
                is_propagate: false,
            };
            Some(analyze_statement(&stmt, scope)?)
        }
        // Check if it's "εἰ" or "ἐάν" (elif)
        else if check_conditional_start(second_expr) {
            // Build a new statement for the elif chain
            let mut elif_clauses = Vec::with_capacity(stmt.clauses().len() - 1);

            // New clause 0: just the elif condition (from clause 1, expr 1)
            elif_clauses.push(Clause {
                expressions: vec![then_clause.expressions[1].clone()],
            });

            // Add remaining clauses (they contain the bodies)
            elif_clauses.extend_from_slice(&stmt.clauses()[2..]);

            let elif_stmt = Statement::Regular {
                clauses: elif_clauses,
                is_query: false,
                is_propagate: false,
            };

            // Recursively parse as a new conditional (which becomes the else body)
            if let Some(elif_analyzed) = parse_conditional(&elif_stmt, scope, depth + 1)? {
                Some(vec![elif_analyzed])
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(Some(AnalyzedStatement::If {
        condition: Box::new(condition_expr),
        then_body,
        else_body,
    }))
}

/// Skip the first word of a clause and parse the rest as an expression
fn skip_first_word_and_parse(
    clause: &Clause,
    scope: &mut Scope,
) -> Result<AnalyzedExpr, GlossaError> {
    // Create a modified clause without the first word
    let mut modified_clause = clause.clone();

    // Remove the first word from the first expression
    if let Some(first_expr) = modified_clause.expressions.first_mut()
        && let Expr::Phrase(terms) = first_expr
        && !terms.is_empty()
    {
        terms.remove(0);
    }

    // Parse the modified clause as a statement
    let stmt = Statement::Regular {
        clauses: vec![modified_clause],
        is_query: false,
        is_propagate: false,
    };
    let analyzed = assemble_statement(&stmt)?;
    let converted = convert_assembled_to_analyzed(&analyzed, scope)?;

    // Extract the first expression as the condition
    match converted {
        AnalyzedStatement::Expression(exprs) => {
            if let Some(first) = exprs.first() {
                Ok(first.clone())
            } else {
                Err(GlossaError::semantic("Empty condition in conditional"))
            }
        }
        AnalyzedStatement::Binding { name, value, .. } => {
            // Convert binding "x is y" to "x == y"
            let left = AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(name),
                glossa_type: value.glossa_type.clone(),
            };
            Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::BinOp {
                    left: Box::new(left),
                    op: crate::morphology::lexicon::BinaryOp::Eq,
                    right: Box::new(value),
                },
                glossa_type: GlossaType::Boolean,
            })
        }
        _ => Err(GlossaError::semantic(format!(
            "Invalid condition format: {:?}",
            converted
        ))),
    }
}

/// Check if a clause starts with "εἰ δὲ μή" (else pattern)
/// ⚡ Bolt Optimization: Avoids heap allocations (`.collect::<Vec<String>>()`, `.join(" ")`)
/// by checking the normalized strings directly from the iterator.
fn check_else_pattern_in_expression(expr: &Expr) -> bool {
    if let Expr::Phrase(terms) = expr {
        let mut words = terms.iter().take(3).filter_map(|term| {
            if let Expr::Word(w) = term {
                Some(w.normalized.as_str())
            } else {
                None
            }
        });

        words.next() == Some(lexicon::ELSE_PATTERN_WORDS[0])
            && words.next() == Some(lexicon::ELSE_PATTERN_WORDS[1])
            && words.next() == Some(lexicon::ELSE_PATTERN_WORDS[2])
    } else {
        false
    }
}

/// Check if an expression starts with a conditional particle (εἰ or ἐάν)
fn check_conditional_start(expr: &Expr) -> bool {
    // Get the first word from the expression
    let first_word = if let Expr::Phrase(terms) = expr {
        terms.first().and_then(|term| {
            if let Expr::Word(w) = term {
                Some(w.normalized.clone())
            } else {
                None
            }
        })
    } else if let Expr::Word(w) = expr {
        Some(w.normalized.clone())
    } else {
        None
    };

    if let Some(word) = first_word {
        lexicon::is_conditional_particle(&word)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Clause, Expr, Statement, Word};

    #[test]
    fn test_for_iteration_error_not_word() {
        let mut scope = Scope::new();

        let stmt = Statement::Regular {
            clauses: vec![
                Clause {
                    expressions: vec![Expr::Phrase(vec![
                        Expr::Word(Word::new("δια")),
                        Expr::NumberLiteral(5),
                    ])],
                },
                Clause {
                    expressions: vec![Expr::Phrase(vec![
                        Expr::Word(Word::new("ν")),
                        Expr::Word(Word::new("λεγε")),
                    ])],
                },
            ],
            is_query: false,
            is_propagate: false,
        };

        let result = analyze_control_flow(&stmt, &mut scope);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Expected word for collection")
        );
    }

    #[test]
    fn test_for_iteration_error_missing_collection() {
        let mut scope = Scope::new();

        let stmt = Statement::Regular {
            clauses: vec![
                Clause {
                    expressions: vec![Expr::Phrase(vec![Expr::Word(Word::new("δια"))])],
                },
                Clause {
                    expressions: vec![Expr::Phrase(vec![
                        Expr::Word(Word::new("ν")),
                        Expr::Word(Word::new("λεγε")),
                    ])],
                },
            ],
            is_query: false,
            is_propagate: false,
        };

        let result = analyze_control_flow(&stmt, &mut scope);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("For iteration needs: διὰ collection")
        );
    }

    #[test]
    fn test_for_iteration_error_not_phrase() {
        let mut scope = Scope::new();

        // This requires testing parse_for_iteration_loop directly or bypassing analyze_control_flow
        // Since analyze_control_flow filters on get_first_word (which expects a Phrase),
        // we call the inner function.
        let stmt = Statement::Regular {
            clauses: vec![
                Clause {
                    expressions: vec![Expr::NumberLiteral(10)], // Not a phrase
                },
                Clause {
                    expressions: vec![Expr::Phrase(vec![
                        Expr::Word(Word::new("ν")),
                        Expr::Word(Word::new("λεγε")),
                    ])],
                },
            ],
            is_query: false,
            is_propagate: false,
        };

        let result = parse_for_iteration_loop(&stmt, &mut scope);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Expected phrase in for iteration")
        );
    }

    #[test]
    fn test_check_else_pattern_not_phrase() {
        let expr = Expr::NumberLiteral(42);
        assert!(!super::check_else_pattern_in_expression(&expr));
    }

    #[test]
    fn test_parse_while_loop_missing_body() {
        let mut scope = Scope::new();

        // ἕως x == 5
        let stmt = Statement::Regular {
            clauses: vec![Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word::new("εως")),
                    Expr::Word(Word::new("ξ")),
                    Expr::Word(Word::new("ισον")),
                    Expr::NumberLiteral(5),
                ])],
            }],
            is_query: false,
            is_propagate: false,
        };

        let result = analyze_control_flow(&stmt, &mut scope);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("While loop needs at least 2 clauses: condition and body")
        );
    }

    #[test]
    fn test_parse_while_loop_success() {
        let mut scope = Scope::new();
        // pre-define ξ so the expression analyzer knows it
        scope.define("ξ".to_string(), GlossaType::Number);

        // ἕως ξ ἴσον 5· «γεια» λέγε
        let stmt = Statement::Regular {
            clauses: vec![
                Clause {
                    expressions: vec![Expr::Phrase(vec![
                        Expr::Word(Word::new("εως")),
                        Expr::Word(Word::new("ξ")),
                        Expr::Word(Word::new("ισον")),
                        Expr::NumberLiteral(5),
                    ])],
                },
                Clause {
                    expressions: vec![Expr::Phrase(vec![
                        Expr::StringLiteral("γεια".to_string()),
                        Expr::Word(Word::new("λεγε")),
                    ])],
                },
            ],
            is_query: false,
            is_propagate: false,
        };

        let result = analyze_control_flow(&stmt, &mut scope);
        assert!(result.is_ok());
        let analyzed = result.unwrap().unwrap();

        match analyzed {
            AnalyzedStatement::While { condition, body } => {
                // Assert condition
                assert_eq!(condition.glossa_type, GlossaType::Boolean);
                // Assert body
                assert!(!body.is_empty());
            }
            _ => panic!("Expected While statement"),
        }
    }

    #[test]
    fn test_parse_for_range_missing_body() {
        let mut scope = Scope::new();

        // ἀπὸ 1 μέχρι 5
        let stmt = Statement::Regular {
            clauses: vec![Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word::new("απο")),
                    Expr::NumberLiteral(1),
                    Expr::Word(Word::new("μεχρι")),
                    Expr::NumberLiteral(5),
                ])],
            }],
            is_query: false,
            is_propagate: false,
        };

        let result = analyze_control_flow(&stmt, &mut scope);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("For loop needs at least 2 clauses: range and body")
        );
    }
}
