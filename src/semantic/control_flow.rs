//! Control flow analysis and parsing
//!
//! Handles if, while, for, match, return, break, continue.

use super::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope, StatementKind,
    analyze_single_statement_with_assembler, convert_assembled_to_analyzed,
};
use crate::ast::{Clause, Expr, Statement};
use crate::errors::GlossaError;
use crate::grammar::normalize_greek;
use crate::morphology::lexicon;
// Circular dependencies handled by crate structure
use super::declarations::parse_function_definition;
use super::expressions::{contains_function_definition_verb, get_first_word};

/// Check if a statement is a control flow construct and analyze it
/// Returns Some(AnalyzedStatement) if it's control flow, None otherwise
pub fn analyze_control_flow(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    // Check for function definition (contains ὁρίζειν)
    if contains_function_definition_verb(stmt) {
        return parse_function_definition(stmt, scope);
    }

    // Get the first word to check for control flow particles
    // If this fails (e.g., statement starts with literal), it's not control flow
    let first_word = match get_first_word(stmt) {
        Ok(word) => word,
        Err(_) => return Ok(None),
    };
    let normalized = normalize_greek(&first_word);

    // Conditional: εἰ/ἐάν condition, body [, εἰ δὲ μή, else_body]
    if lexicon::is_conditional_particle(&normalized) {
        return parse_conditional(stmt, scope);
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
        return Ok(Some(AnalyzedStatement {
            kind: StatementKind::Break,
            expressions: vec![],
        }));
    }

    // Continue: συνέχιζε
    if lexicon::is_continue_verb(&normalized) {
        return Ok(Some(AnalyzedStatement {
            kind: StatementKind::Continue,
            expressions: vec![],
        }));
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

    let body_analyzed = if let Some(cf) = analyze_control_flow(&body_stmt, scope)? {
        cf
    } else {
        let assembled = analyze_single_statement_with_assembler(&body_stmt)?;
        convert_assembled_to_analyzed(&assembled, scope)?
    };

    Ok(Some(AnalyzedStatement {
        kind: StatementKind::While {
            condition: Box::new(condition_expr),
            body: vec![body_analyzed],
        },
        expressions: vec![],
    }))
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
    // Pattern: ἀπὸ start μέχρι/ἕως end
    // Structure in clause.expressions[0].Phrase: [ἀπὸ, start, μέχρι/ἕως, end]
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
    let start_expr = if let Expr::Word(w) = &words[1] {
        // Parse as number literal
        if let Some(val) = lexicon::numeral_value(&w.normalized) {
            AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(val),
                glossa_type: GlossaType::Number,
            }
        } else {
            // Try as variable
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(w.normalized.clone()),
                glossa_type: scope
                    .lookup(&w.normalized)
                    .cloned()
                    .unwrap_or(GlossaType::Number),
            }
        }
    } else {
        return Err(GlossaError::semantic("Expected word for range start"));
    };

    // Check if it's inclusive (ἕως) or exclusive (μέχρι) - word at index 2
    let inclusive = if let Expr::Word(w) = &words[2] {
        normalize_greek(&w.original) == "εως"
    } else {
        false
    };

    // Extract end (word at index 3)
    let end_expr = if let Expr::Word(w) = &words[3] {
        // Parse as number literal
        if let Some(val) = lexicon::numeral_value(&w.normalized) {
            AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(val),
                glossa_type: GlossaType::Number,
            }
        } else {
            // Try as variable
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(w.normalized.clone()),
                glossa_type: scope
                    .lookup(&w.normalized)
                    .cloned()
                    .unwrap_or(GlossaType::Number),
            }
        }
    } else {
        return Err(GlossaError::semantic("Expected word for range end"));
    };

    let iterator = AnalyzedExpr {
        expr: AnalyzedExprKind::Range {
            start: Box::new(start_expr),
            end: Box::new(end_expr),
            inclusive,
        },
        glossa_type: GlossaType::Number,
    };

    // Parse body - all remaining clauses form the body
    // Body might be a single clause or multiple clauses (e.g., if statement)
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

    // Parse body as a multi-clause statement (handles if/else/etc)
    let body_stmt = Statement::Regular {
        clauses: body_clauses.to_vec(),
        is_query: false,
        is_propagate: false,
    };

    // Add loop variable to scope temporarily while parsing body
    scope.define(variable.clone(), GlossaType::Number);

    // Check if body is control flow, otherwise parse as regular statement
    let body_analyzed = if let Some(cf) = analyze_control_flow(&body_stmt, scope)? {
        cf
    } else {
        let assembled = analyze_single_statement_with_assembler(&body_stmt)?;
        convert_assembled_to_analyzed(&assembled, scope)?
    };

    // Note: We don't remove the variable from scope since Scope doesn't support that
    // In a real implementation, we'd use nested scopes

    Ok(Some(AnalyzedStatement {
        kind: StatementKind::For {
            variable,
            iterator: Box::new(iterator),
            body: vec![body_analyzed],
        },
        expressions: vec![],
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
    // Pattern: διὰ στοιχείων -> extract "στοιχείων" as collection variable
    let collection_clause = &stmt.clauses()[0];

    if collection_clause.expressions.is_empty() {
        return Err(GlossaError::semantic("Empty collection clause in for loop"));
    }

    let collection_name = if let Expr::Phrase(terms) = &collection_clause.expressions[0] {
        // Skip διά (first word) and get the collection name (second word)
        if terms.len() < 2 {
            return Err(GlossaError::semantic("For iteration needs: διὰ collection"));
        }
        if let Expr::Word(w) = &terms[1] {
            w.normalized.clone()
        } else {
            return Err(GlossaError::semantic("Expected word for collection"));
        }
    } else {
        return Err(GlossaError::semantic("Expected phrase in for iteration"));
    };

    // Create a variable expression for the collection
    // TODO: Look up the collection in scope to get its actual type
    let collection_type = scope
        .lookup(&collection_name)
        .cloned()
        .unwrap_or(GlossaType::String);
    let collection_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::Variable(collection_name),
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

    // Parse body as multi-clause statement
    let body_stmt = Statement::Regular {
        clauses: body_clauses.to_vec(),
        is_query: false,
        is_propagate: false,
    };

    // Add loop variable to scope temporarily
    // TODO: Infer element type from collection type
    scope.define(variable.clone(), GlossaType::String);

    let body_analyzed = if let Some(cf) = analyze_control_flow(&body_stmt, scope)? {
        cf
    } else {
        let assembled = analyze_single_statement_with_assembler(&body_stmt)?;
        convert_assembled_to_analyzed(&assembled, scope)?
    };

    Ok(Some(AnalyzedStatement {
        kind: StatementKind::For {
            variable,
            iterator: Box::new(collection_expr),
            body: vec![body_analyzed],
        },
        expressions: vec![],
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
    let scrutinee = skip_first_word_and_parse(scrutinee_clause, scope)?;

    // Build arms: pattern from clause[i], expr 1 → body from clause[i+1], expr 0
    let mut arms = Vec::new();

    for i in 0..stmt.clauses().len() {
        let clause = &stmt.clauses()[i];

        // Get pattern from expression 1 (if it exists)
        if clause.expressions.len() > 1 {
            let pattern_expr = &clause.expressions[1];

            // Extract pattern value (skip ᾖ subjunctive verb)
            // Pattern is the first word(s) before ᾖ
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
            let body_stmt = parse_clause_as_statement(&body_expr_clause, scope)?;

            arms.push((pattern, vec![body_stmt]));
        }
    }

    if arms.is_empty() {
        return Err(GlossaError::semantic(
            "Match expression needs at least one arm",
        ));
    }

    Ok(Some(AnalyzedStatement {
        kind: StatementKind::Match {
            scrutinee: Box::new(scrutinee),
            arms,
        },
        expressions: vec![],
    }))
}

/// Parse a return statement: δός value
fn parse_return_statement(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    // Get the first clause
    if stmt.clauses().is_empty() {
        return Ok(Some(AnalyzedStatement {
            kind: StatementKind::Return { value: None },
            expressions: vec![],
        }));
    }

    let clause = &stmt.clauses()[0];

    // Try to parse return expression - for now, use a simple heuristic
    // Full expression parsing will need a context-aware approach
    let return_expr = parse_return_expression(clause, scope)?;

    Ok(Some(AnalyzedStatement {
        kind: StatementKind::Return {
            value: Some(Box::new(return_expr)),
        },
        expressions: vec![],
    }))
}

/// Parse return expression in a simple way (avoiding assembler issues with scoped variables)
fn parse_return_expression(clause: &Clause, scope: &Scope) -> Result<AnalyzedExpr, GlossaError> {
    // For Cycle 3, we'll do simple expression parsing
    // The expression after δός could be:
    // - A simple variable: ξ
    // - A literal: πέντε
    // - An operation: ξ δύο ἄθροισμα

    // Get all words after δός
    let words = if let Some(Expr::Phrase(terms)) = clause.expressions.first() {
        terms.iter().skip(1).collect::<Vec<_>>() // Skip δός
    } else {
        vec![]
    };

    if words.is_empty() {
        return Ok(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(0),
            glossa_type: GlossaType::Number,
        });
    }

    // Simple heuristic: if single word or literal
    if words.len() == 1 {
        match words[0] {
            Expr::Word(w) => {
                let normalized = normalize_greek(&w.original);

                // Check if it's a numeral
                if let Some(val) = lexicon::numeral_value(&normalized) {
                    return Ok(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(val),
                        glossa_type: GlossaType::Number,
                    });
                }

                // Treat as variable
                let var_type = scope
                    .lookup(&normalized)
                    .cloned()
                    .unwrap_or(GlossaType::Unknown);
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(normalized),
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
/// Patterns can be: literals (μηδὲν, ἓν, δύο), or wildcard (ἄλλο)
fn parse_match_pattern(expr: &Expr, scope: &mut Scope) -> Result<AnalyzedExpr, GlossaError> {
    // Pattern is typically: value ᾖ
    // We want to extract the value part
    if let Expr::Phrase(terms) = expr {
        if terms.is_empty() {
            return Err(GlossaError::semantic("Empty match pattern"));
        }

        // Get first word (the pattern value)
        if let Expr::Word(w) = &terms[0] {
            let normalized = normalize_greek(&w.original);

            // Check if it's ἄλλο (wildcard)
            if normalized == "αλλο" {
                // Wildcard pattern - represented as a special marker
                // We'll use a boolean literal true as a placeholder for wildcard
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                });
            }

            // Check if it's a numeral
            if let Some(val) = lexicon::numeral_value(&normalized) {
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(val),
                    glossa_type: GlossaType::Number,
                });
            }

            // Otherwise, treat as variable reference
            let var_type = scope
                .lookup(&normalized)
                .cloned()
                .unwrap_or(GlossaType::Number);
            return Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(normalized),
                glossa_type: var_type,
            });
        }
    } else if let Expr::Word(w) = expr {
        let normalized = normalize_greek(&w.original);

        // Check for wildcard
        if normalized == "αλλο" {
            return Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: GlossaType::Boolean,
            });
        }

        // Check for numeral
        if let Some(val) = lexicon::numeral_value(&normalized) {
            return Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(val),
                glossa_type: GlossaType::Number,
            });
        }
    }

    Err(GlossaError::semantic("Invalid match pattern"))
}

/// Parse a conditional statement (εἰ/ἐάν)
/// Structure: εἰ condition, body [, εἰ δὲ μή, else_body]
fn parse_conditional(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if stmt.clauses().len() < 2 {
        return Err(GlossaError::semantic(
            "Conditional needs at least 2 clauses: condition and body",
        ));
    }

    // Parse condition (first clause, minus the εἰ/ἐάν particle)
    let condition_clause = &stmt.clauses()[0];
    let condition_expr = skip_first_word_and_parse(condition_clause, scope)?;

    // Parse then-body (second clause, first expression)
    // Note: Clause 1 may have multiple expressions chained with middle dot (·)
    // The first expression is the then-body, the second (if present) is the else marker
    let then_clause = &stmt.clauses()[1];
    let then_body = if then_clause.expressions.is_empty() {
        return Err(GlossaError::semantic("Empty then-body in conditional"));
    } else {
        // Create a mini-clause with just the first expression
        let first_expr_clause = Clause {
            expressions: vec![then_clause.expressions[0].clone()],
        };
        parse_clause_as_statement(&first_expr_clause, scope)?
    };

    // Check for else/elif clause
    // The second expression in clause 1 (if present) can be:
    // 1. "εἰ δὲ μή" - else clause
    // 2. "εἰ" - elif chain (nested if)
    let else_body = if then_clause.expressions.len() > 1 && stmt.clauses().len() >= 3 {
        let second_expr = &then_clause.expressions[1];

        // Check if it's "εἰ δὲ μή" (else)
        if check_else_pattern_in_expression(second_expr) {
            Some(vec![parse_clause_as_statement(&stmt.clauses()[2], scope)?])
        }
        // Check if it's "εἰ" or "ἐάν" (elif)
        else if check_conditional_start(second_expr) {
            // Build a new statement for the elif chain
            // The elif condition is in clause 1, expression 1
            // We need to restructure: make a new clause 0 with just that expression
            let mut elif_clauses = Vec::new();

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
            if let Some(elif_analyzed) = parse_conditional(&elif_stmt, scope)? {
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

    Ok(Some(AnalyzedStatement {
        kind: StatementKind::If {
            condition: Box::new(condition_expr),
            then_body: vec![then_body],
            else_body,
        },
        expressions: vec![], // Control flow doesn't use flat expression list
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
    let stmt = parse_clause_as_mini_statement(&modified_clause)?;
    let analyzed = analyze_single_statement_with_assembler(&stmt)?;
    let converted = convert_assembled_to_analyzed(&analyzed, scope)?;

    // Extract the first expression as the condition
    if let Some(first) = converted.expressions.first() {
        Ok(first.clone())
    } else {
        Err(GlossaError::semantic("Empty condition in conditional"))
    }
}

/// Parse a clause as a statement
fn parse_clause_as_statement(
    clause: &Clause,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    let stmt = parse_clause_as_mini_statement(clause)?;

    // Check if it's control flow first (break, continue, etc.)
    if let Some(cf) = analyze_control_flow(&stmt, scope)? {
        return Ok(cf);
    }

    let assembled = analyze_single_statement_with_assembler(&stmt)?;
    convert_assembled_to_analyzed(&assembled, scope)
}

/// Convert a clause to a mini-statement for parsing
fn parse_clause_as_mini_statement(clause: &Clause) -> Result<Statement, GlossaError> {
    Ok(Statement::Regular {
        clauses: vec![clause.clone()],
        is_query: false,
        is_propagate: false,
    })
}

/// Check if a clause starts with "εἰ δὲ μή" (else pattern)
/// Check if an expression matches "εἰ δὲ μή" (else pattern)
fn check_else_pattern_in_expression(expr: &Expr) -> bool {
    // Extract first 3 words from the expression
    let words: Vec<String> = if let Expr::Phrase(terms) = expr {
        terms
            .iter()
            .take(3)
            .filter_map(|term| {
                if let Expr::Word(w) = term {
                    Some(normalize_greek(&w.original).to_string())
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![]
    };

    let phrase = words.join(" ");
    lexicon::is_else_pattern(&phrase)
}

/// Check if an expression starts with a conditional particle (εἰ or ἐάν)
fn check_conditional_start(expr: &Expr) -> bool {
    // Get the first word from the expression
    let first_word = if let Expr::Phrase(terms) = expr {
        terms.first().and_then(|term| {
            if let Expr::Word(w) = term {
                Some(normalize_greek(&w.original))
            } else {
                None
            }
        })
    } else if let Expr::Word(w) = expr {
        Some(normalize_greek(&w.original))
    } else {
        None
    };

    if let Some(word) = first_word {
        lexicon::is_conditional_particle(&word)
    } else {
        false
    }
}
