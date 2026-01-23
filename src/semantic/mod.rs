//! Semantic analysis for ΓΛΩΣΣΑ
//!
//! This module handles:
//! - Slot-based sentence assembly (Greek-native word-order independence)
//! - Name resolution and scope tracking
//! - Gender/number/case agreement checking
//! - Type inference from morphology
//!
//! ## The Assembler Approach
//!
//! Unlike traditional parsers that rely on word position, ΓΛΩΣΣΑ uses a
//! slot-based assembler that routes words to grammatical slots based on
//! their case endings - just like Ancient Greek actually works.
//!
//! ```text
//! Nominative → Subject slot
//! Accusative → Object slot
//! Dative     → Indirect object slot
//! Genitive   → Possession/attachment
//! ```
//!
//! This means SOV, VSO, and OVS all produce the same result!

mod resolver;
mod agreement;
mod types;
pub mod assembler;
pub mod disambiguation;

pub use resolver::*;
pub use agreement::*;
pub use types::*;
pub use assembler::{Assembler, AssembledStatement, AssemblyError, Constituent, VerbConstituent, Literal};
pub use disambiguation::{DisambiguationContext, disambiguate, resolve_best, analyze_article};

use crate::ast::{Program, Statement, Expr};
use crate::errors::GlossaError;
use crate::morphology;
use crate::grammar::normalize_greek;

/// Perform semantic analysis on a program using the slot-based assembler
/// This is the primary entry point that provides word-order independence
pub fn analyze_program(program: &Program) -> Result<AnalyzedProgram, GlossaError> {
    let mut scope = Scope::new();
    let mut analyzed_statements = Vec::new();

    for stmt in &program.statements {
        // Check if this is a control flow construct
        if let Some(control_flow_stmt) = analyze_control_flow(stmt, &mut scope)? {
            analyzed_statements.push(control_flow_stmt);
        } else {
            // Use the assembler-based approach for regular statements
            let assembled = analyze_single_statement_with_assembler(stmt)?;
            let analyzed = convert_assembled_to_analyzed(&assembled, &mut scope)?;
            analyzed_statements.push(analyzed);
        }
    }

    Ok(AnalyzedProgram {
        statements: analyzed_statements,
        scope,
    })
}

/// Legacy analysis method that doesn't use the assembler
/// Kept for comparison and fallback purposes
pub fn analyze_program_legacy(program: &Program) -> Result<AnalyzedProgram, GlossaError> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(program)
}

/// Analyze a single statement using the slot-based assembler
fn analyze_single_statement_with_assembler(stmt: &Statement) -> Result<AssembledStatement, GlossaError> {
    let mut asm = Assembler::new();
    asm.set_query(stmt.is_query);

    // Disambiguation context accumulator - articles set context for following words
    let mut current_context = DisambiguationContext::new();

    // Feed each expression/term to the assembler with disambiguation
    // Process all clauses - they're separated by commas in the grammar
    for clause in &stmt.clauses {
        for expr in &clause.expressions {
            feed_expr_to_assembler_with_context(&mut asm, expr, &mut current_context)?;
        }
    }

    // Finalize the statement
    match asm.finalize() {
        Ok(assembled) => Ok(assembled),
        Err(e) => Err(GlossaError::semantic(&e.to_string())),
    }
}

/// Check if a statement is a control flow construct and analyze it
/// Returns Some(AnalyzedStatement) if it's control flow, None otherwise
fn analyze_control_flow(stmt: &Statement, _scope: &mut Scope) -> Result<Option<AnalyzedStatement>, GlossaError> {
    use crate::morphology::lexicon;

    // Get the first word to check for control flow particles
    // If this fails (e.g., statement starts with literal), it's not control flow
    let first_word = match get_first_word(stmt) {
        Ok(word) => word,
        Err(_) => return Ok(None),
    };
    let normalized = normalize_greek(&first_word);

    // Conditional: εἰ/ἐάν condition, body [, εἰ δὲ μή, else_body]
    if lexicon::is_conditional_particle(&normalized) {
        return parse_conditional(stmt, _scope);
    }

    // While loop: ἕως condition, body
    if normalized == "εως" {
        return parse_while_loop(stmt, _scope);
    }

    // For loop with range: ἀπὸ start μέχρι/ἕως end, body
    if normalized == "απο" {
        return parse_for_range_loop(stmt, _scope);
    }

    // For loop with iteration: διὰ collection, body
    if normalized == "δια" {
        return parse_for_iteration_loop(stmt, _scope);
    }

    if lexicon::is_loop_particle(&normalized) {
        // Other loop forms handled above
        return Ok(None);
    }

    if lexicon::is_match_particle(&normalized) {
        // TODO: Parse match (κατά)
        return Ok(None);
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

/// Get the first word from a statement for pattern detection
fn get_first_word(stmt: &Statement) -> Result<String, GlossaError> {
    if let Some(first_clause) = stmt.clauses.first() {
        if let Some(first_expr) = first_clause.expressions.first() {
            if let Expr::Phrase(terms) = first_expr {
                if let Some(first_term) = terms.first() {
                    if let Expr::Word(word) = first_term {
                        return Ok(word.original.clone());
                    }
                }
            } else if let Expr::Word(word) = first_expr {
                return Ok(word.original.clone());
            }
        }
    }
    Err(GlossaError::semantic("Empty statement"))
}

/// Parse a while loop statement (ἕως)
/// Structure: ἕως condition, body
fn parse_while_loop(stmt: &Statement, scope: &mut Scope) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if stmt.clauses.len() < 2 {
        return Err(GlossaError::semantic("While loop needs at least 2 clauses: condition and body"));
    }

    // Parse condition (first clause, minus the ἕως particle)
    let condition_clause = &stmt.clauses[0];
    let condition_expr = skip_first_word_and_parse(condition_clause, scope)?;

    // Parse body - all remaining clauses
    let body_clauses = &stmt.clauses[1..];

    let body_stmt = Statement {
        clauses: body_clauses.to_vec(),
        is_query: false,
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
fn parse_for_range_loop(stmt: &Statement, scope: &mut Scope) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if stmt.clauses.len() < 2 {
        return Err(GlossaError::semantic("For loop needs at least 2 clauses: range and body"));
    }

    // Parse range specification from first clause
    // Pattern: ἀπὸ start μέχρι/ἕως end
    // Structure in clause.expressions[0].Phrase: [ἀπὸ, start, μέχρι/ἕως, end]
    let range_clause = &stmt.clauses[0];

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
        return Err(GlossaError::semantic("For range needs: ἀπὸ start μέχρι/ἕως end"));
    }

    // Extract start (word at index 1)
    let start_expr = if let Expr::Word(w) = &words[1] {
        // Parse as number literal
        if let Some(val) = crate::morphology::lexicon::numeral_value(&w.normalized) {
            AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(val),
                glossa_type: GlossaType::Number,
            }
        } else {
            // Try as variable
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(w.normalized.clone()),
                glossa_type: scope.lookup(&w.normalized).cloned().unwrap_or(GlossaType::Number),
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
        if let Some(val) = crate::morphology::lexicon::numeral_value(&w.normalized) {
            AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(val),
                glossa_type: GlossaType::Number,
            }
        } else {
            // Try as variable
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(w.normalized.clone()),
                glossa_type: scope.lookup(&w.normalized).cloned().unwrap_or(GlossaType::Number),
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
    let body_clauses = &stmt.clauses[1..];

    if body_clauses.is_empty() {
        return Err(GlossaError::semantic("For loop needs a body"));
    }

    // Extract variable name from first word of first body clause
    let variable = if let Some(first_expr) = body_clauses[0].expressions.first() {
        if let Expr::Phrase(terms) = first_expr {
            if let Some(Expr::Word(w)) = terms.first() {
                w.normalized.clone()
            } else {
                "i".to_string()
            }
        } else if let Expr::Word(w) = first_expr {
            w.normalized.clone()
        } else {
            "i".to_string()
        }
    } else {
        "i".to_string()
    };

    // Parse body as a multi-clause statement (handles if/else/etc)
    let body_stmt = Statement {
        clauses: body_clauses.to_vec(),
        is_query: false,
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
fn parse_for_iteration_loop(stmt: &Statement, scope: &mut Scope) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if stmt.clauses.len() < 2 {
        return Err(GlossaError::semantic("For loop needs at least 2 clauses: collection and body"));
    }

    // Extract collection name from first clause (διὰ + genitive noun)
    // Pattern: διὰ στοιχείων -> extract "στοιχείων" as collection variable
    let collection_clause = &stmt.clauses[0];

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
    let collection_type = scope.lookup(&collection_name).cloned().unwrap_or(GlossaType::String);
    let collection_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::Variable(collection_name),
        glossa_type: collection_type,
    };

    // Parse body - all remaining clauses
    let body_clauses = &stmt.clauses[1..];

    // Extract variable name from first word of body
    let variable = if let Some(first_expr) = body_clauses[0].expressions.first() {
        if let Expr::Phrase(terms) = first_expr {
            if let Some(Expr::Word(w)) = terms.first() {
                w.normalized.clone()
            } else {
                "x".to_string()
            }
        } else if let Expr::Word(w) = first_expr {
            w.normalized.clone()
        } else {
            "x".to_string()
        }
    } else {
        "x".to_string()
    };

    // Parse body as multi-clause statement
    let body_stmt = Statement {
        clauses: body_clauses.to_vec(),
        is_query: false,
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

/// Parse a conditional statement (εἰ/ἐάν)
/// Structure: εἰ condition, body [, εἰ δὲ μή, else_body]
fn parse_conditional(stmt: &Statement, scope: &mut Scope) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if stmt.clauses.len() < 2 {
        return Err(GlossaError::semantic("Conditional needs at least 2 clauses: condition and body"));
    }

    // Parse condition (first clause, minus the εἰ/ἐάν particle)
    let condition_clause = &stmt.clauses[0];
    let condition_expr = skip_first_word_and_parse(condition_clause, scope)?;

    // Parse then-body (second clause, first expression)
    // Note: Clause 1 may have multiple expressions chained with middle dot (·)
    // The first expression is the then-body, the second (if present) is the else marker
    let then_clause = &stmt.clauses[1];
    let then_body = if then_clause.expressions.is_empty() {
        return Err(GlossaError::semantic("Empty then-body in conditional"));
    } else {
        // Create a mini-clause with just the first expression
        let first_expr_clause = crate::ast::Clause {
            expressions: vec![then_clause.expressions[0].clone()],
        };
        parse_clause_as_statement(&first_expr_clause, scope)?
    };

    // Check for else clause (εἰ δὲ μή)
    // The else marker should be the second expression in clause 1, chained with ·
    // The else body is in clause 2
    let else_body = if then_clause.expressions.len() > 1 && stmt.clauses.len() >= 3 {
        // Check if the second expression in clause 1 is "εἰ δὲ μή"
        if check_else_pattern_in_expression(&then_clause.expressions[1]) {
            Some(vec![parse_clause_as_statement(&stmt.clauses[2], scope)?])
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
fn skip_first_word_and_parse(clause: &crate::ast::Clause, scope: &mut Scope) -> Result<AnalyzedExpr, GlossaError> {
    // Create a modified clause without the first word
    let mut modified_clause = clause.clone();

    // Remove the first word from the first expression
    if let Some(first_expr) = modified_clause.expressions.first_mut() {
        if let Expr::Phrase(terms) = first_expr {
            if !terms.is_empty() {
                terms.remove(0);
            }
        }
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
fn parse_clause_as_statement(clause: &crate::ast::Clause, scope: &mut Scope) -> Result<AnalyzedStatement, GlossaError> {
    let stmt = parse_clause_as_mini_statement(clause)?;

    // Check if it's control flow first (break, continue, etc.)
    if let Some(cf) = analyze_control_flow(&stmt, scope)? {
        return Ok(cf);
    }

    let assembled = analyze_single_statement_with_assembler(&stmt)?;
    convert_assembled_to_analyzed(&assembled, scope)
}

/// Convert a clause to a mini-statement for parsing
fn parse_clause_as_mini_statement(clause: &crate::ast::Clause) -> Result<Statement, GlossaError> {
    Ok(Statement {
        clauses: vec![clause.clone()],
        is_query: false,
    })
}

/// Check if a clause starts with "εἰ δὲ μή" (else pattern)
fn check_else_pattern(clause: &crate::ast::Clause) -> bool {
    use crate::morphology::lexicon;

    // Collect first 3 words and check if they form "εἰ δὲ μή"
    let words: Vec<String> = clause.expressions.iter()
        .take(3)
        .filter_map(|expr| {
            if let Expr::Phrase(terms) = expr {
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
            }
        })
        .collect();

    let phrase = words.join(" ");
    lexicon::is_else_pattern(&phrase)
}

/// Check if an expression matches "εἰ δὲ μή" (else pattern)
fn check_else_pattern_in_expression(expr: &Expr) -> bool {
    use crate::morphology::lexicon;

    // Extract first 3 words from the expression
    let words: Vec<String> = if let Expr::Phrase(terms) = expr {
        terms.iter()
            .take(3)
            .filter_map(|term| {
                if let Expr::Word(w) = term {
                    Some(normalize_greek(&w.original))
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

/// Feed an expression into the assembler with disambiguation context
fn feed_expr_to_assembler_with_context(
    asm: &mut Assembler,
    expr: &Expr,
    context: &mut DisambiguationContext,
) -> Result<(), GlossaError> {
    match expr {
        Expr::StringLiteral(s) => {
            asm.feed_string(s.clone());
        }
        Expr::NumberLiteral(n) => {
            asm.feed_number(*n);
        }
        Expr::BooleanLiteral(b) => {
            asm.feed_boolean(*b);
        }
        Expr::Word(w) => {
            // Check if this is an article using ORIGINAL form (preserves diacritics)
            // This distinguishes ἡ (article) from ἤ (or) - they differ only in breathing/accent
            if let Some(article_context) = analyze_article(&w.original) {
                *context = article_context;
                // Articles themselves don't go to assembler slots
                return Ok(());
            }

            // Get all possible analyses for the word
            let analyses = morphology::analyze_all(&w.normalized);

            // Use disambiguation context to pick the best analysis
            let best_analysis = resolve_best(analyses, context);

            // Feed the disambiguated analysis to assembler
            if let Err(e) = asm.feed(&best_analysis, &w.original) {
                return Err(GlossaError::semantic(&e.to_string()));
            }

            // Clear context after use (it was consumed by the following noun)
            *context = DisambiguationContext::new();
        }
        Expr::Phrase(terms) => {
            // Feed each term in the phrase, passing context through
            for term in terms {
                feed_expr_to_assembler_with_context(asm, term, context)?;
            }
        }
        Expr::PropertyAccess { owner, property } => {
            // Owner is genitive, property is what it attaches to
            feed_expr_to_assembler_with_context(asm, owner, context)?;
            feed_expr_to_assembler_with_context(asm, property, context)?;
        }
        Expr::Call { verb, arguments } => {
            // Feed the verb - verbs can set context for subjects
            let analyses = morphology::analyze_all(&verb.normalized);
            let best_verb = resolve_best(analyses, context);

            // Set context from verb for potential subject agreement
            *context = DisambiguationContext::from_verb(&best_verb);

            if let Err(e) = asm.feed(&best_verb, &verb.original) {
                return Err(GlossaError::semantic(&e.to_string()));
            }

            // Feed arguments
            for arg in arguments {
                feed_expr_to_assembler_with_context(asm, arg, context)?;
            }
        }
        Expr::Binding { name, value } => {
            // Feed the name and value (binding verbs handled by assembler)
            let analyses = morphology::analyze_all(&name.normalized);
            let best_name = resolve_best(analyses, context);

            if let Err(e) = asm.feed(&best_name, &name.original) {
                return Err(GlossaError::semantic(&e.to_string()));
            }
            feed_expr_to_assembler_with_context(asm, value, context)?;
        }
        Expr::BinOp { left, op: _, right } => {
            // TODO: Implement binary operation handling
            feed_expr_to_assembler_with_context(asm, left, context)?;
            feed_expr_to_assembler_with_context(asm, right, context)?;
        }
        Expr::UnaryOp { op: _, operand } => {
            // TODO: Implement unary operation handling
            feed_expr_to_assembler_with_context(asm, operand, context)?;
        }
        Expr::Block(statements) => {
            // TODO: Handle block expressions (for control flow bodies)
            // For now, just process statements recursively
            for stmt in statements {
                for clause in &stmt.clauses {
                    for expr in &clause.expressions {
                        feed_expr_to_assembler_with_context(asm, expr, context)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Convert an AssembledStatement to an AnalyzedStatement
/// This bridges the slot-based assembler output to the HIR lowering input
fn convert_assembled_to_analyzed(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    // Determine statement kind based on assembled content
    let (kind, expressions) = classify_assembled_statement(asm_stmt, scope)?;

    Ok(AnalyzedStatement { kind, expressions })
}

/// Classify an assembled statement and extract analyzed expressions
fn classify_assembled_statement(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<(StatementKind, Vec<AnalyzedExpr>), GlossaError> {
    // Check for binding pattern: has subject + literals + binding verb
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);
        if crate::morphology::lexicon::is_binding_verb(&verb_lemma) {
            // Check if this is actually a comparison with subjunctive (εἰ condition)
            // Pattern: subject operator literal subjunctive-verb
            if !asm_stmt.operators.is_empty() &&
               asm_stmt.literals.len() >= 1 &&
               verb.mood == Some(crate::morphology::Mood::Subjunctive) {
                // This is a comparison expression, not a binding
                // Build: subject op literal
                if let Some(ref subject) = asm_stmt.subject {
                    // Get left operand (subject variable)
                    let left = if let Some(var_type) = scope.lookup(&subject.lemma) {
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(subject.lemma.clone()),
                            glossa_type: var_type.clone(),
                        }
                    } else {
                        // Variable not in scope, treat as boolean literal false
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::BooleanLiteral(false),
                            glossa_type: GlossaType::Boolean,
                        }
                    };

                    // Get right operand (first literal)
                    let right = literal_to_analyzed_expr(&asm_stmt.literals[0]);

                    // Build binary expression
                    let op = asm_stmt.operators[0];
                    let comparison = build_binary_expr(left, op, right);

                    return Ok((StatementKind::Expression, vec![comparison]));
                }
            }

            // Binding: subject is the variable name, literals are the value
            if let Some(ref subject) = asm_stmt.subject {
                let name = subject.lemma.clone();

                // Get value from literals or object
                let (value_expr, value_type) = extract_value(asm_stmt);

                // Register binding in scope
                scope.define(name.clone(), value_type.clone());

                return Ok((
                    StatementKind::Binding {
                        name: name.clone(),
                        value_type: value_type.clone(),
                    },
                    vec![
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(name),
                            glossa_type: value_type.clone(),
                        },
                        value_expr,
                    ],
                ));
            }
        }

        // Check for print pattern
        if crate::morphology::lexicon::is_print_verb(&verb_lemma) {
            // If we have operators, combine subject/variables with literals using operators
            if !asm_stmt.operators.is_empty() {
                // Get left operand (subject variable)
                let left = if let Some(ref subj) = asm_stmt.subject {
                    if let Some(var_type) = scope.lookup(&subj.lemma) {
                        Some(AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                            glossa_type: var_type.clone(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Get right operand (first literal)
                let right = asm_stmt.literals.first().map(literal_to_analyzed_expr);

                // If we have both operands and an operator, build a binary expression
                if let (Some(left_expr), Some(right_expr)) = (left, right) {
                    let op = asm_stmt.operators[0];
                    let bin_expr = build_binary_expr(left_expr, op, right_expr);
                    return Ok((StatementKind::Print, vec![bin_expr]));
                }
            }

            // Build binary expressions from literals and operators if available
            // This handles cases like: true || false
            let mut args = build_expressions_from_literals_and_ops(
                &asm_stmt.literals,
                &asm_stmt.operators,
            );

            // Also include subject/object if present (variable references)
            if let Some(ref subj) = asm_stmt.subject {
                if let Some(var_type) = scope.lookup(&subj.lemma) {
                    args.insert(0, AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                        glossa_type: var_type.clone(),
                    });
                }
            }

            if let Some(ref obj) = asm_stmt.object {
                if let Some(var_type) = scope.lookup(&obj.lemma) {
                    args.push(AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                        glossa_type: var_type.clone(),
                    });
                }
            }

            return Ok((StatementKind::Print, args));
        }
    }

    // Query pattern
    if asm_stmt.is_query {
        let mut exprs = Vec::new();
        for lit in &asm_stmt.literals {
            exprs.push(literal_to_analyzed_expr(lit));
        }
        if let Some(ref subj) = asm_stmt.subject {
            let var_type = scope.lookup(&subj.lemma).cloned().unwrap_or(GlossaType::Unknown);
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: var_type,
            });
        }
        return Ok((StatementKind::Query, exprs));
    }

    // Default: expression statement
    let exprs = build_expressions_from_literals_and_ops(
        &asm_stmt.literals,
        &asm_stmt.operators,
    );

    Ok((StatementKind::Expression, exprs))
}

/// Extract value from assembled statement (literals or constituents)
fn extract_value(asm_stmt: &AssembledStatement) -> (AnalyzedExpr, GlossaType) {
    // If we have operators, build a binary expression from literals
    if !asm_stmt.operators.is_empty() && asm_stmt.literals.len() >= 2 {
        let exprs = build_expressions_from_literals_and_ops(
            &asm_stmt.literals,
            &asm_stmt.operators,
        );
        if let Some(expr) = exprs.into_iter().next() {
            let ty = expr.glossa_type.clone();
            return (expr, ty);
        }
    }

    // Prefer literals (single value, no operators)
    if let Some(lit) = asm_stmt.literals.first() {
        return (literal_to_analyzed_expr(lit), literal_to_type(lit));
    }

    // Otherwise use object
    if let Some(ref obj) = asm_stmt.object {
        // Check if it's a numeral word
        if let Some(value) = crate::morphology::lexicon::numeral_value(&normalize_greek(&obj.lemma)) {
            return (
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(value),
                    glossa_type: GlossaType::Number,
                },
                GlossaType::Number,
            );
        }

        return (
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            },
            GlossaType::Unknown,
        );
    }

    // Default
    (
        AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(0),
            glossa_type: GlossaType::Number,
        },
        GlossaType::Number,
    )
}

/// Convert a Literal to an AnalyzedExpr
fn literal_to_analyzed_expr(lit: &Literal) -> AnalyzedExpr {
    match lit {
        Literal::String(s) => AnalyzedExpr {
            expr: AnalyzedExprKind::StringLiteral(s.clone()),
            glossa_type: GlossaType::String,
        },
        Literal::Number(n) => AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(*n),
            glossa_type: GlossaType::Number,
        },
        Literal::Boolean(b) => AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(*b),
            glossa_type: GlossaType::Boolean,
        },
    }
}

/// Get the type of a Literal
fn literal_to_type(lit: &Literal) -> GlossaType {
    match lit {
        Literal::String(_) => GlossaType::String,
        Literal::Number(_) => GlossaType::Number,
        Literal::Boolean(_) => GlossaType::Boolean,
    }
}

/// Build a binary expression from two analyzed expressions and an operator
fn build_binary_expr(
    left: AnalyzedExpr,
    op: crate::morphology::lexicon::BinaryOp,
    right: AnalyzedExpr,
) -> AnalyzedExpr {
    let result_type = infer_binop_type(&left.glossa_type, &op, &right.glossa_type);
    AnalyzedExpr {
        expr: AnalyzedExprKind::BinOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        },
        glossa_type: result_type,
    }
}

/// Build expressions from literals and operators
/// If there are operators, builds a binary expression tree
/// Otherwise, returns the literals as-is
fn build_expressions_from_literals_and_ops(
    literals: &[Literal],
    operators: &[crate::morphology::lexicon::BinaryOp],
) -> Vec<AnalyzedExpr> {
    // If no operators, just return literals as separate expressions
    if operators.is_empty() {
        return literals.iter().map(literal_to_analyzed_expr).collect();
    }

    // If we have operators, build a binary expression
    // Pattern: lit0 op0 lit1 op1 lit2 ... -> ((lit0 op0 lit1) op1 lit2) ...
    if literals.len() < 2 || operators.is_empty() {
        return literals.iter().map(literal_to_analyzed_expr).collect();
    }

    // Build left-associative tree
    let mut result = literal_to_analyzed_expr(&literals[0]);

    for (i, op) in operators.iter().enumerate() {
        if i + 1 < literals.len() {
            let right = literal_to_analyzed_expr(&literals[i + 1]);
            let result_type = infer_binop_type(&result.glossa_type, op, &right.glossa_type);
            result = AnalyzedExpr {
                expr: AnalyzedExprKind::BinOp {
                    left: Box::new(result),
                    op: *op,
                    right: Box::new(right),
                },
                glossa_type: result_type,
            };
        }
    }

    vec![result]
}

/// Infer the result type of a binary operation
fn infer_binop_type(
    _left: &GlossaType,
    op: &crate::morphology::lexicon::BinaryOp,
    _right: &GlossaType,
) -> GlossaType {
    use crate::morphology::lexicon::BinaryOp;

    match op {
        // Arithmetic operations on numbers return numbers
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
            GlossaType::Number
        }
        // Comparison operations return booleans
        BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            GlossaType::Boolean
        }
        // Boolean operations return booleans
        BinaryOp::And | BinaryOp::Or => {
            GlossaType::Boolean
        }
    }
}

/// Analyzed program with resolved names and types
#[derive(Debug, Clone)]
pub struct AnalyzedProgram {
    pub statements: Vec<AnalyzedStatement>,
    pub scope: Scope,
}

/// Analyzed statement
#[derive(Debug, Clone)]
pub struct AnalyzedStatement {
    pub kind: StatementKind,
    pub expressions: Vec<AnalyzedExpr>,
}

/// The kind of statement
#[derive(Debug, Clone)]
pub enum StatementKind {
    /// Variable binding: ξ πέντε ἔστω
    Binding { name: String, value_type: GlossaType },
    /// Print statement: «χαῖρε» λέγε
    Print,
    /// Expression statement
    Expression,
    /// Query: ξ?
    Query,
    /// If conditional: εἰ condition, body [εἰ δὲ μή, else_body]
    If {
        condition: Box<AnalyzedExpr>,
        then_body: Vec<AnalyzedStatement>,
        else_body: Option<Vec<AnalyzedStatement>>,
    },
    /// While loop: ἕως condition, body
    While {
        condition: Box<AnalyzedExpr>,
        body: Vec<AnalyzedStatement>,
    },
    /// For loop: διά/ἀπό...μέχρι
    For {
        variable: String,
        iterator: Box<AnalyzedExpr>,
        body: Vec<AnalyzedStatement>,
    },
    /// Match expression: κατά scrutinee { arms }
    Match {
        scrutinee: Box<AnalyzedExpr>,
        arms: Vec<(AnalyzedExpr, Vec<AnalyzedStatement>)>,
    },
    /// Break: παῦε
    Break,
    /// Continue: συνέχιζε
    Continue,
}

/// Analyzed expression with type information
#[derive(Debug, Clone)]
pub struct AnalyzedExpr {
    pub expr: AnalyzedExprKind,
    pub glossa_type: GlossaType,
}

/// Kind of analyzed expression
#[derive(Debug, Clone)]
pub enum AnalyzedExprKind {
    StringLiteral(String),
    NumberLiteral(i64),
    BooleanLiteral(bool),
    Variable(String),
    PropertyAccess { owner: Box<AnalyzedExpr>, property: String },
    VerbCall { verb: String, args: Vec<AnalyzedExpr> },
    /// Binary operation (arithmetic, comparison, boolean)
    BinOp {
        left: Box<AnalyzedExpr>,
        op: crate::morphology::lexicon::BinaryOp,
        right: Box<AnalyzedExpr>,
    },
    /// Unary operation (negation)
    UnaryOp {
        op: crate::morphology::lexicon::UnaryOp,
        operand: Box<AnalyzedExpr>,
    },
    /// Range expression for loops (start..end or start..=end)
    Range {
        start: Box<AnalyzedExpr>,
        end: Box<AnalyzedExpr>,
        inclusive: bool,
    },
}

/// Semantic analyzer state
pub struct SemanticAnalyzer {
    scope: Scope,
    errors: Vec<GlossaError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            scope: Scope::new(),
            errors: Vec::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<AnalyzedProgram, GlossaError> {
        let mut analyzed_statements = Vec::new();

        for stmt in &program.statements {
            match self.analyze_statement(stmt) {
                Ok(analyzed) => analyzed_statements.push(analyzed),
                Err(e) => self.errors.push(e),
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }

        Ok(AnalyzedProgram {
            statements: analyzed_statements,
            scope: self.scope.clone(),
        })
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<AnalyzedStatement, GlossaError> {
        // Determine statement kind by analyzing expressions
        // Collect expressions from all clauses
        let exprs: Vec<&Expr> = stmt.expressions().collect();

        if exprs.is_empty() {
            return Ok(AnalyzedStatement {
                kind: StatementKind::Expression,
                expressions: vec![],
            });
        }

        // Analyze the first (and usually only) expression
        let first_expr = exprs[0];
        let (kind, analyzed_exprs) = self.analyze_expression_list(first_expr, stmt.is_query)?;

        Ok(AnalyzedStatement {
            kind,
            expressions: analyzed_exprs,
        })
    }

    fn analyze_expression_list(
        &mut self,
        expr: &Expr,
        is_query: bool,
    ) -> Result<(StatementKind, Vec<AnalyzedExpr>), GlossaError> {
        match expr {
            Expr::Phrase(terms) => {
                self.analyze_phrase(terms, is_query)
            }
            _ => {
                let analyzed = self.analyze_single_expr(expr)?;
                let kind = if is_query {
                    StatementKind::Query
                } else {
                    StatementKind::Expression
                };
                Ok((kind, vec![analyzed]))
            }
        }
    }

    fn analyze_phrase(
        &mut self,
        terms: &[Expr],
        is_query: bool,
    ) -> Result<(StatementKind, Vec<AnalyzedExpr>), GlossaError> {
        // Look for patterns: binding, print, property access

        // Find verb position and type
        let mut verb_idx = None;
        let mut is_binding = false;
        let mut is_print = false;

        for (i, term) in terms.iter().enumerate() {
            if let Expr::Word(w) = term {
                let normalized = &w.normalized;
                if crate::morphology::lexicon::is_binding_verb(normalized) {
                    verb_idx = Some(i);
                    is_binding = true;
                    break;
                }
                if crate::morphology::lexicon::is_print_verb(normalized) {
                    verb_idx = Some(i);
                    is_print = true;
                    break;
                }
            }
        }

        if is_binding {
            // Pattern: name value ἔστω
            // e.g., ξ πέντε ἔστω
            if terms.len() >= 3 {
                let name = self.extract_name(&terms[0])?;
                let value = self.analyze_single_expr(&terms[1])?;

                // Register in scope
                self.scope.define(name.clone(), value.glossa_type.clone());

                return Ok((
                    StatementKind::Binding {
                        name: name.clone(),
                        value_type: value.glossa_type.clone(),
                    },
                    vec![
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(name),
                            glossa_type: value.glossa_type.clone(),
                        },
                        value,
                    ],
                ));
            }
        }

        if is_print {
            // Pattern: value λέγε
            let mut args = Vec::new();
            for term in terms.iter().take(verb_idx.unwrap_or(terms.len())) {
                args.push(self.analyze_single_expr(term)?);
            }

            return Ok((
                StatementKind::Print,
                args,
            ));
        }

        // Default: analyze all terms
        let mut analyzed = Vec::new();
        for term in terms {
            analyzed.push(self.analyze_single_expr(term)?);
        }

        let kind = if is_query {
            StatementKind::Query
        } else {
            StatementKind::Expression
        };

        Ok((kind, analyzed))
    }

    fn analyze_single_expr(&self, expr: &Expr) -> Result<AnalyzedExpr, GlossaError> {
        match expr {
            Expr::StringLiteral(s) => Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::StringLiteral(s.clone()),
                glossa_type: GlossaType::String,
            }),

            Expr::NumberLiteral(n) => Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(*n),
                glossa_type: GlossaType::Number,
            }),

            Expr::BooleanLiteral(b) => Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(*b),
                glossa_type: GlossaType::Boolean,
            }),

            Expr::Word(w) => {
                // Check if it's a numeral word
                if let Some(value) = crate::morphology::lexicon::numeral_value(&w.normalized) {
                    return Ok(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(value),
                        glossa_type: GlossaType::Number,
                    });
                }

                // Check if it's a known variable
                if let Some(var_type) = self.scope.lookup(&w.normalized) {
                    return Ok(AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(w.normalized.clone()),
                        glossa_type: var_type.clone(),
                    });
                }

                // Unknown word - treat as variable reference
                Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(w.normalized.clone()),
                    glossa_type: GlossaType::Unknown,
                })
            }

            Expr::Phrase(terms) => {
                // Nested phrase - analyze recursively
                if terms.len() == 1 {
                    return self.analyze_single_expr(&terms[0]);
                }

                // For now, return first term's type
                let first = self.analyze_single_expr(&terms[0])?;
                Ok(first)
            }

            _ => Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("_".to_string()),
                glossa_type: GlossaType::Unknown,
            }),
        }
    }

    fn extract_name(&self, expr: &Expr) -> Result<String, GlossaError> {
        match expr {
            Expr::Word(w) => Ok(w.normalized.clone()),
            _ => Err(GlossaError::semantic("Expected a word for variable name")),
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::build_ast;

    #[test]
    fn test_analyze_hello() {
        let ast = build_ast("«χαῖρε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        assert_eq!(analyzed.statements.len(), 1);
        assert!(matches!(analyzed.statements[0].kind, StatementKind::Print));
    }

    #[test]
    fn test_analyze_binding() {
        let ast = build_ast("ξ πέντε ἔστω.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        assert!(matches!(
            &analyzed.statements[0].kind,
            StatementKind::Binding { name, .. } if name == "ξ"
        ));

        // Check that ξ is now in scope
        assert!(analyzed.scope.lookup("ξ").is_some());
    }

    #[test]
    fn test_analyze_variable_use() {
        let ast = build_ast("ξ πέντε ἔστω. ξ λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        assert_eq!(analyzed.statements.len(), 2);
        // Second statement should reference ξ with known type
        assert!(matches!(analyzed.statements[1].kind, StatementKind::Print));
    }

    #[test]
    fn test_analyze_string_literal() {
        let ast = build_ast("«χαῖρε κόσμε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        let first_expr = &analyzed.statements[0].expressions[0];
        assert_eq!(first_expr.glossa_type, GlossaType::String);
    }

    #[test]
    fn test_analyze_number_literal() {
        let ast = build_ast("42 λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        let first_expr = &analyzed.statements[0].expressions[0];
        assert_eq!(first_expr.glossa_type, GlossaType::Number);
    }
}
