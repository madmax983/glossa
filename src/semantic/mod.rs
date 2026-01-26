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
        // Handle type definitions
        if let Statement::TypeDefinition(type_def) = stmt {
            analyzed_statements.push(analyze_type_definition(type_def, &mut scope)?);
            continue;
        }

        // Check if this is a control flow construct
        if let Some(control_flow_stmt) = analyze_control_flow(stmt, &mut scope)? {
            // If it's a function definition, register it in the scope
            if let StatementKind::FunctionDef { name, params, return_type, .. } = &control_flow_stmt.kind {
                let param_types: Vec<GlossaType> = params.iter()
                    .map(|(_, ty)| ty.clone().unwrap_or(GlossaType::Unknown))
                    .collect();
                scope.define_function(name.clone(), param_types, return_type.clone());
            }
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
    asm.set_query(stmt.is_query());

    // Disambiguation context accumulator - articles set context for following words
    let mut current_context = DisambiguationContext::new();

    // Feed each expression/term to the assembler with disambiguation
    // Process all clauses - they're separated by commas in the grammar
    for clause in stmt.clauses() {
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

    // Check for function definition (contains ὁρίζειν)
    if contains_function_definition_verb(stmt) {
        return parse_function_definition(stmt, _scope);
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
        return parse_match_expression(stmt, _scope);
    }

    // Return: δός (give)
    if normalized == "δος" {
        return parse_return_statement(stmt, _scope);
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

/// Try to parse a struct instantiation: variable νέον type_name args ἔστω
/// Analyze a type definition statement
fn analyze_type_definition(type_def: &crate::ast::TypeDef, scope: &mut Scope) -> Result<AnalyzedStatement, GlossaError> {
    use crate::grammar::normalize_greek;

    // Extract type name
    let type_name = normalize_greek(&type_def.name.original);

    // Analyze fields
    let mut fields = Vec::new();
    for field in &type_def.fields {
        let field_name = normalize_greek(&field.name.original);
        let type_name_gen = normalize_greek(&field.type_name.original);

        // Map genitive type names to GlossaType
        let field_type = map_genitive_to_type(&type_name_gen, scope);
        fields.push((field_name, field_type));
    }

    // Create the struct type
    let struct_type = GlossaType::Struct {
        name: type_name.clone(),
        gender: crate::morphology::Gender::Neuter, // Default for now
        fields: fields.clone(),
    };

    // Store the type in scope
    scope.define_type(type_name.clone(), struct_type);

    Ok(AnalyzedStatement {
        kind: StatementKind::TypeDefinition {
            name: type_name,
            fields,
        },
        expressions: vec![],
    })
}

/// Map a genitive type name to a GlossaType
fn map_genitive_to_type(genitive_name: &str, scope: &Scope) -> GlossaType {
    // ἀριθμοῦ (genitive of ἀριθμός) → Number
    if genitive_name == "αριθμου" {
        return GlossaType::Number;
    }
    // ὀνόματος (genitive of ὄνομα) → String
    if genitive_name == "ονοματος" {
        return GlossaType::String;
    }
    // Check for user-defined types
    // Strip genitive ending and look up the nominative form
    // For now, just try the name as-is
    if let Some(ty) = scope.lookup_type(genitive_name) {
        return ty.clone();
    }

    GlossaType::Unknown
}

/// Analyze an argument expression (could be literal, variable, or nested call)
fn analyze_argument_expr(expr: &Expr, scope: &Scope) -> Result<AnalyzedExpr, GlossaError> {
    match expr {
        Expr::Word(w) => {
            let normalized = normalize_greek(&w.original);

            // Check if it's a numeral
            if let Some(val) = crate::morphology::lexicon::numeral_value(&normalized) {
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(val),
                    glossa_type: GlossaType::Number,
                });
            }

            // Check if it's a variable
            if let Some(var_type) = scope.lookup(&normalized) {
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(normalized),
                    glossa_type: var_type.clone(),
                });
            }

            // Unknown variable
            Err(GlossaError::semantic(&format!("Undefined variable: {}", normalized)))
        }

        Expr::NumberLiteral(n) => {
            Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(*n),
                glossa_type: GlossaType::Number,
            })
        }

        Expr::StringLiteral(s) => {
            Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::StringLiteral(s.clone()),
                glossa_type: GlossaType::String,
            })
        }

        Expr::BooleanLiteral(b) => {
            Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(*b),
                glossa_type: GlossaType::Boolean,
            })
        }

        Expr::Phrase(terms) => {
            // A phrase could be a function call: function_name arg1 arg2 ...
            if terms.is_empty() {
                return Err(GlossaError::semantic("Empty phrase in argument"));
            }

            // Check if first term is a function name
            if let Expr::Word(w) = &terms[0] {
                let func_name = normalize_greek(&w.original);

                if scope.is_function(&func_name) {
                    // It's a function call - recursively analyze arguments
                    let mut args = Vec::new();
                    for arg_expr in &terms[1..] {
                        args.push(analyze_argument_expr(arg_expr, scope)?);
                    }

                    let return_type = scope.lookup_function(&func_name)
                        .and_then(|sig| sig.return_type.clone())
                        .unwrap_or(GlossaType::Unknown);

                    return Ok(AnalyzedExpr {
                        expr: AnalyzedExprKind::FunctionCall {
                            func: func_name,
                            args,
                        },
                        glossa_type: return_type,
                    });
                }
            }

            // Not a function call - could be a complex expression
            // For now, just analyze the first term
            analyze_argument_expr(&terms[0], scope)
        }

        Expr::Block(statements) => {
            // Parenthesized expression - analyze as nested expression
            // Extract the expression from the block
            if let Some(stmt) = statements.first() {
                if let Some(clause) = stmt.clauses().first() {
                    if let Some(expr) = clause.expressions.first() {
                        return analyze_argument_expr(expr, scope);
                    }
                }
            }
            Err(GlossaError::semantic("Empty or invalid block expression"))
        }

        _ => Err(GlossaError::semantic("Unsupported argument expression type")),
    }
}

/// Get the first word from a statement for pattern detection
fn get_first_word(stmt: &Statement) -> Result<String, GlossaError> {
    if let Some(first_clause) = stmt.clauses().first() {
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

/// Check if a statement contains the function definition verb (ὁρίζειν)
fn contains_function_definition_verb(stmt: &Statement) -> bool {
    for clause in stmt.clauses() {
        for expr in &clause.expressions {
            if contains_verb_in_expr(expr, "οριζειν") {
                return true;
            }
        }
    }
    false
}

/// Helper to check if an expression contains a specific verb
fn contains_verb_in_expr(expr: &Expr, verb: &str) -> bool {
    match expr {
        Expr::Word(word) => normalize_greek(&word.original) == verb,
        Expr::Phrase(terms) => terms.iter().any(|t| contains_verb_in_expr(t, verb)),
        _ => false,
    }
}

/// Parse a function definition: name ὁρίζειν [params]· body
fn parse_function_definition(stmt: &Statement, scope: &mut Scope) -> Result<Option<AnalyzedStatement>, GlossaError> {
    // The middle dot (·) separates expressions within a clause
    // Structure: expr1 · expr2 where expr1 is "name ὁρίζειν [params]" and expr2 is the body

    // Get the first clause
    if stmt.clauses().is_empty() {
        return Err(GlossaError::semantic("Function definition cannot be empty"));
    }

    let clause = &stmt.clauses()[0];

    // We need at least 2 expressions: header and body
    if clause.expressions.len() < 2 {
        return Err(GlossaError::semantic("Function definition needs header and body separated by middle dot (·)"));
    }

    // Parse header (first expression contains name, ὁρίζειν, and parameters)
    let header_expr = &clause.expressions[0];
    let function_name = extract_function_name_from_expr(header_expr)?;

    // Extract parameters (dative words) from the header
    let params = extract_parameters_from_expr(header_expr)?;

    // Create a new scope for the function body and add parameters to it
    let mut function_scope = Scope::new();
    for (param_name, param_type) in &params {
        let glossa_type = param_type.clone().unwrap_or(GlossaType::Unknown);
        function_scope.define(param_name.clone(), glossa_type);
    }

    // Parse body (remaining expressions after middle dot)
    let body_exprs = &clause.expressions[1..];
    let mut body_statements = Vec::new();

    // Each expression in the body becomes a statement
    for expr in body_exprs {
        // Create a clause with this single expression
        let body_clause = crate::ast::Clause {
            expressions: vec![expr.clone()],
        };

        let clause_stmt = Statement::Regular {
            clauses: vec![body_clause],
            is_query: false,
        };

        // Analyze each body expression as a statement with the function scope
        if let Some(cf) = analyze_control_flow(&clause_stmt, &mut function_scope)? {
            body_statements.push(cf);
        } else {
            let assembled = analyze_single_statement_with_assembler(&clause_stmt)?;
            let analyzed = convert_assembled_to_analyzed(&assembled, &mut function_scope)?;
            body_statements.push(analyzed);
        }
    }

    // Infer return type from return statements
    let return_type = infer_return_type_from_body(&body_statements);

    Ok(Some(AnalyzedStatement {
        kind: StatementKind::FunctionDef {
            name: function_name,
            params,
            body: body_statements,
            return_type,
        },
        expressions: vec![],
    }))
}

/// Extract the function name from a function definition header expression
fn extract_function_name_from_expr(expr: &Expr) -> Result<String, GlossaError> {
    // Collect all words and find the nominative word before ὁρίζειν
    let words = collect_words_from_expr(expr);

    let mut function_name = None;

    for (i, word) in words.iter().enumerate() {
        let normalized = normalize_greek(&word.original);

        // Stop at ὁρίζειν
        if normalized == "οριζειν" {
            if let Some(name) = function_name {
                return Ok(name);
            } else {
                break;
            }
        }

        // Look for nominative words (or words without clear case marking, which default to nominative)
        let analysis = morphology::analyze(&word.original);
        if analysis.part_of_speech != morphology::PartOfSpeech::Article
            && analysis.part_of_speech != morphology::PartOfSpeech::Verb
        {
            // This could be the function name
            function_name = Some(normalized);
        }
    }

    function_name.ok_or_else(|| GlossaError::semantic("Could not find function name in definition"))
}

/// Extract parameters from a function definition header expression
/// Parameters are words after dative article τῷ, optionally followed by genitive type annotations
fn extract_parameters_from_expr(expr: &Expr) -> Result<Vec<(String, Option<GlossaType>)>, GlossaError> {
    let mut params = Vec::new();

    // Collect all words from the expression
    let words = collect_words_from_expr(expr);

    // Find the position of ὁρίζειν
    let mut start_pos = None;
    for (i, word) in words.iter().enumerate() {
        let normalized = normalize_greek(&word.original);
        if normalized == "οριζειν" {
            start_pos = Some(i + 1); // Start after ὁρίζειν
            break;
        }
    }

    let start_pos = start_pos.unwrap_or(0);
    let mut i = start_pos;

    while i < words.len() {
        let word = &words[i];
        let analysis = morphology::analyze(&word.original);
        let normalized = normalize_greek(&word.original);

        // Check for dative article τῷ
        if analysis.part_of_speech == morphology::PartOfSpeech::Article
            && analysis.case == Some(morphology::Case::Dative)
        {
            // Next word should be the parameter name
            if i + 1 < words.len() {
                let param_word = &words[i + 1];
                let param_name = normalize_greek(&param_word.original);

                // Check if word after param is a genitive (type annotation)
                let mut param_type = None;
                if i + 2 < words.len() {
                    let next_word = &words[i + 2];
                    let next_analysis = morphology::analyze(&next_word.original);
                    if next_analysis.case == Some(morphology::Case::Genitive) {
                        // Map genitive type to GlossaType
                        let type_name = normalize_greek(&next_word.original);
                        param_type = Some(map_greek_type_to_glossa(&type_name));
                        i += 1; // Skip the type annotation
                    }
                }

                params.push((param_name, param_type));
                i += 1; // Skip the parameter name
            }
        }

        i += 1;
    }

    Ok(params)
}

/// Collect all Word nodes from an expression (flattening phrases)
fn collect_words_from_expr(expr: &Expr) -> Vec<&crate::ast::Word> {
    let mut words = Vec::new();

    match expr {
        Expr::Word(word) => words.push(word),
        Expr::Phrase(terms) => {
            for term in terms {
                words.extend(collect_words_from_expr(term));
            }
        }
        _ => {}
    }

    words
}

/// Map Greek type names to GlossaType
fn map_greek_type_to_glossa(type_name: &str) -> GlossaType {
    match type_name {
        "αριθμου" => GlossaType::Number,
        "ονοματος" => GlossaType::String,
        "λιστης" => GlossaType::List(Box::new(GlossaType::Unknown)),
        _ => GlossaType::Unknown,
    }
}

/// Infer the return type from the function body by examining return statements
fn infer_return_type_from_body(body: &[AnalyzedStatement]) -> Option<GlossaType> {
    for stmt in body {
        if let StatementKind::Return { value } = &stmt.kind {
            if let Some(return_expr) = value {
                return Some(return_expr.glossa_type.clone());
            }
        }
    }
    None
}

/// Parse a while loop statement (ἕως)
/// Structure: ἕως condition, body
fn parse_while_loop(stmt: &Statement, scope: &mut Scope) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if stmt.clauses().len() < 2 {
        return Err(GlossaError::semantic("While loop needs at least 2 clauses: condition and body"));
    }

    // Parse condition (first clause, minus the ἕως particle)
    let condition_clause = &stmt.clauses()[0];
    let condition_expr = skip_first_word_and_parse(condition_clause, scope)?;

    // Parse body - all remaining clauses
    let body_clauses = &stmt.clauses()[1..];

    let body_stmt = Statement::Regular {
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
    if stmt.clauses().len() < 2 {
        return Err(GlossaError::semantic("For loop needs at least 2 clauses: range and body"));
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
    let body_stmt = Statement::Regular {
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
    if stmt.clauses().len() < 2 {
        return Err(GlossaError::semantic("For loop needs at least 2 clauses: collection and body"));
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
    let collection_type = scope.lookup(&collection_name).cloned().unwrap_or(GlossaType::String);
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
    let body_stmt = Statement::Regular {
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

/// Parse a match expression (κατά)
/// Structure: κατὰ scrutinee· pattern₁ ᾖ, body₁· pattern₂ ᾖ, body₂· pattern₃ ᾖ, body₃.
/// Example: κατὰ ξ· μηδὲν ᾖ, «μηδέν» λέγε· ἓν ᾖ, «ἕν» λέγε· ἄλλο ᾖ, «ἄλλο» λέγε.
fn parse_match_expression(stmt: &Statement, scope: &mut Scope) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if stmt.clauses().is_empty() {
        return Err(GlossaError::semantic("Match expression needs at least one clause"));
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
            let body_expr_clause = crate::ast::Clause {
                expressions: vec![body_clause.expressions[0].clone()],
            };
            let body_stmt = parse_clause_as_statement(&body_expr_clause, scope)?;

            arms.push((pattern, vec![body_stmt]));
        }
    }

    if arms.is_empty() {
        return Err(GlossaError::semantic("Match expression needs at least one arm"));
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
fn parse_return_statement(stmt: &Statement, scope: &mut Scope) -> Result<Option<AnalyzedStatement>, GlossaError> {
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
fn parse_return_expression(clause: &crate::ast::Clause, scope: &Scope) -> Result<AnalyzedExpr, GlossaError> {
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

    // Simple heuristic: if single word, treat as variable or literal
    if words.len() == 1 {
        if let Expr::Word(w) = words[0] {
            let normalized = normalize_greek(&w.original);

            // Check if it's a numeral
            if let Some(val) = crate::morphology::lexicon::numeral_value(&normalized) {
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(val),
                    glossa_type: GlossaType::Number,
                });
            }

            // Treat as variable
            let var_type = scope.lookup(&normalized).cloned().unwrap_or(GlossaType::Unknown);
            return Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(normalized),
                glossa_type: var_type,
            });
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
            if let Some(val) = crate::morphology::lexicon::numeral_value(&normalized) {
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(val),
                    glossa_type: GlossaType::Number,
                });
            }

            // Otherwise, treat as variable reference
            let var_type = scope.lookup(&normalized).cloned().unwrap_or(GlossaType::Number);
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
        if let Some(val) = crate::morphology::lexicon::numeral_value(&normalized) {
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
fn parse_conditional(stmt: &Statement, scope: &mut Scope) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if stmt.clauses().len() < 2 {
        return Err(GlossaError::semantic("Conditional needs at least 2 clauses: condition and body"));
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
        let first_expr_clause = crate::ast::Clause {
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
            elif_clauses.push(crate::ast::Clause {
                expressions: vec![then_clause.expressions[1].clone()],
            });

            // Add remaining clauses (they contain the bodies)
            elif_clauses.extend_from_slice(&stmt.clauses()[2..]);

            let elif_stmt = Statement::Regular {
                clauses: elif_clauses,
                is_query: false,
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
    Ok(Statement::Regular {
        clauses: vec![clause.clone()],
        is_query: false,
    })
}

/// Check if a clause starts with "εἰ δὲ μή" (else pattern)
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

/// Check if an expression starts with a conditional particle (εἰ or ἐάν)
fn check_conditional_start(expr: &Expr) -> bool {
    use crate::morphology::lexicon;

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
            // But detect nested phrases (parenthesized expressions) and store them separately
            for term in terms {
                if matches!(term, Expr::Phrase(_)) {
                    // This is a nested phrase (parenthesized expression)
                    // Store it for later analysis instead of flattening
                    if let Expr::Phrase(nested_terms) = term {
                        asm.feed_nested_phrase(nested_terms.clone());
                    }
                } else {
                    feed_expr_to_assembler_with_context(asm, term, context)?;
                }
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
            // Parenthesized expressions are stored as blocks for later analysis
            // Don't feed their contents to the main assembler - they'll be analyzed separately
            asm.feed_block(statements.clone());
        }
        Expr::ArrayLiteral(elements) => {
            // Feed array literal to assembler
            asm.feed_array(elements.clone());
        }
        Expr::IndexAccess { array, index } => {
            // Feed index access to assembler
            asm.feed_index_access(array.as_ref().clone(), index.as_ref().clone());
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
    // Check for property access pattern: genitive + nominative + verb
    // Pattern: genitive_var nominative_field λέγε
    // Example: που ξ λέγε → pi.xi
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        if crate::morphology::lexicon::is_print_verb(&verb_lemma) &&
           !asm_stmt.genitives.is_empty() &&
           asm_stmt.subject.is_some() {
            // Get owner from genitive (use lemma to get base variable name)
            let owner_lemma = &asm_stmt.genitives[0].lemma;

            // Get property from subject (nominative)
            let property = normalize_greek(&asm_stmt.subject.as_ref().unwrap().original);

            // Check if owner is a struct type in scope
            if let Some(owner_type) = scope.lookup(owner_lemma) {
                if matches!(owner_type, GlossaType::Struct { .. }) {
                    // Build property access
                    let prop_access = AnalyzedExpr {
                        expr: AnalyzedExprKind::PropertyAccess {
                            owner: Box::new(AnalyzedExpr {
                                expr: AnalyzedExprKind::Variable(owner_lemma.clone()),
                                glossa_type: owner_type.clone(),
                            }),
                            property: property.clone(),
                        },
                        glossa_type: GlossaType::Unknown, // TODO: Look up field type
                    };

                    return Ok((StatementKind::Print, vec![prop_access]));
                }
            }
        }
    }

    // Check for struct instantiation pattern
    // Pattern: subject νέον type_name args... ἔστω
    // Example: π νέον σημεῖον πέντε ἔστω
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        if crate::morphology::lexicon::is_binding_verb(&verb_lemma) &&
           !asm_stmt.adjectives.is_empty() &&
           asm_stmt.subject.is_some() &&
           asm_stmt.object.is_some() {
            // Check if adjective is νέον (new)
            let adj_lemma = normalize_greek(&asm_stmt.adjectives[0].lemma);
            if adj_lemma == "νεος" {
                // Get variable name from subject
                let var_name = normalize_greek(&asm_stmt.subject.as_ref().unwrap().original);

                // Get type name from object
                let type_name = normalize_greek(&asm_stmt.object.as_ref().unwrap().original);

                // Check if type exists in scope
                if let Some(struct_type) = scope.lookup_type(&type_name).cloned() {
                    // Get constructor arguments from literals
                    let args: Vec<AnalyzedExpr> = asm_stmt.literals.iter()
                        .map(literal_to_analyzed_expr)
                        .collect();

                    // Build struct instantiation
                    let struct_inst = AnalyzedExpr {
                        expr: AnalyzedExprKind::StructInstantiation {
                            type_name: type_name.clone(),
                            args,
                        },
                        glossa_type: struct_type.clone(),
                    };

                    // Register variable in scope
                    scope.define(var_name.clone(), struct_type.clone());

                    return Ok((
                        StatementKind::Binding {
                            name: var_name.clone(),
                            value_type: struct_type.clone(),
                        },
                        vec![
                            AnalyzedExpr {
                                expr: AnalyzedExprKind::Variable(var_name),
                                glossa_type: struct_type.clone(),
                            },
                            struct_inst,
                        ],
                    ));
                }
            }
        }
    }

    // Check for function call pattern
    // Pattern: subject function_name args... ἔστω
    // Where function_name is not a built-in verb but a user-defined function
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        // Check if verb is a binding verb and if there's an object or genitive that could be a function call
        if crate::morphology::lexicon::is_binding_verb(&verb_lemma) {
            // Check if object is a user-defined function
            // Pattern: subject = function(args)
            // The function name might be in the object slot or genitives

            // Try nominatives first (function names are nominative)
            let mut func_name = None;
            for nominative in &asm_stmt.nominatives {
                if scope.is_function(&nominative.lemma) {
                    func_name = Some(nominative.lemma.clone());
                    break;
                }
            }

            // Try object slot if not found in nominatives
            if func_name.is_none() {
                if let Some(ref object) = asm_stmt.object {
                    if scope.is_function(&object.lemma) {
                        func_name = Some(object.lemma.clone());
                    }
                }
            }

            // Try genitives if still not found
            if func_name.is_none() {
                for genitive in &asm_stmt.genitives {
                    if scope.is_function(&genitive.lemma) {
                        func_name = Some(genitive.lemma.clone());
                        break;
                    }
                }
            }

            // If we found a function name, build the call
            if let Some(func) = func_name {
                if let Some(ref subject) = asm_stmt.subject {
                    // Build function call arguments from literals and blocks
                    let mut args: Vec<AnalyzedExpr> = asm_stmt.literals.iter()
                        .map(literal_to_analyzed_expr)
                        .collect();

                    // Add nested function calls from nested phrases (parenthesized expressions)
                    for nested_terms in &asm_stmt.nested_phrases {
                        let phrase_expr = Expr::Phrase(nested_terms.clone());
                        let analyzed = analyze_argument_expr(&phrase_expr, scope)?;
                        args.push(analyzed);
                    }

                    // Get return type from function signature
                    let return_type = scope.lookup_function(&func)
                        .and_then(|sig| sig.return_type.clone())
                        .unwrap_or(GlossaType::Unknown);

                    let func_call = AnalyzedExpr {
                        expr: AnalyzedExprKind::FunctionCall {
                            func: func.clone(),
                            args,
                        },
                        glossa_type: return_type.clone(),
                    };

                    // Register subject as variable (use original form, not lemma)
                    let var_name = normalize_greek(&subject.original);
                    scope.define(var_name.clone(), return_type.clone());

                    return Ok((
                        StatementKind::Binding {
                            name: var_name.clone(),
                            value_type: return_type.clone(),
                        },
                        vec![
                            AnalyzedExpr {
                                expr: AnalyzedExprKind::Variable(var_name),
                                glossa_type: return_type.clone(),
                            },
                            func_call,
                        ],
                    ));
                }
            }
        }

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
            // BUT: check for ambiguous case where subject/object might be swapped
            // Heuristic: if subject is in scope and object is not, they're probably swapped
            let (var_name, actual_asm) = if let (Some(subject), Some(object)) = (&asm_stmt.subject, &asm_stmt.object) {
                let subject_name = normalize_greek(&subject.original);
                let object_name = normalize_greek(&object.original);

                if scope.is_defined(&subject.lemma) && !scope.is_defined(&object.lemma) {
                    // Subject is in scope, object is not → they're swapped
                    // Create a new asm_stmt with swapped subject/object
                    let mut swapped = asm_stmt.clone();
                    swapped.subject = Some(object.clone());
                    swapped.object = Some(subject.clone());
                    (object_name, swapped)
                } else {
                    (subject_name, asm_stmt.clone())
                }
            } else if let Some(subject) = &asm_stmt.subject {
                (normalize_greek(&subject.original), asm_stmt.clone())
            } else {
                return Err(GlossaError::semantic("Binding without subject"));
            };

            // Get value from literals or object
            let (value_expr, value_type) = extract_value(&actual_asm);

            // Register binding in scope
            scope.define(var_name.clone(), value_type.clone());

            return Ok((
                StatementKind::Binding {
                    name: var_name.clone(),
                    value_type: value_type.clone(),
                },
                vec![
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(var_name),
                        glossa_type: value_type.clone(),
                    },
                    value_expr,
                ],
            ));
        }

        // Check for pop pattern: subject ἕλκεται (middle voice)
        if crate::morphology::lexicon::is_pop_verb(&verb_lemma) {
            if let Some(ref subject) = asm_stmt.subject {
                // Get the receiver (array variable)
                let receiver = AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subject.lemma.clone()),
                    glossa_type: scope.lookup(&subject.lemma).cloned().unwrap_or(GlossaType::Unknown),
                };

                // Return method call expression with no arguments
                let method_call = AnalyzedExpr {
                    expr: AnalyzedExprKind::MethodCall {
                        receiver: Box::new(receiver),
                        method: "pop".to_string(),
                        args: vec![],
                    },
                    glossa_type: GlossaType::Unknown, // Option<T>
                };

                return Ok((StatementKind::Expression, vec![method_call]));
            }
        }

        // Check for push pattern: subject ὠθεῖ value
        if crate::morphology::lexicon::is_push_verb(&verb_lemma) {
            if let Some(ref subject) = asm_stmt.subject {
                // Get the receiver (array variable)
                let receiver = AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subject.lemma.clone()),
                    glossa_type: scope.lookup(&subject.lemma).cloned().unwrap_or(GlossaType::Unknown),
                };

                // Get the argument to push (from literals or object)
                let arg = if let Some(lit) = asm_stmt.literals.first() {
                    literal_to_analyzed_expr(lit)
                } else if let Some(ref obj) = asm_stmt.object {
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                        glossa_type: scope.lookup(&obj.lemma).cloned().unwrap_or(GlossaType::Unknown),
                    }
                } else {
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(0),
                        glossa_type: GlossaType::Number,
                    }
                };

                // Return method call expression
                let method_call = AnalyzedExpr {
                    expr: AnalyzedExprKind::MethodCall {
                        receiver: Box::new(receiver),
                        method: "push".to_string(),
                        args: vec![arg],
                    },
                    glossa_type: GlossaType::Unit,
                };

                return Ok((StatementKind::Expression, vec![method_call]));
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

            // Check if we have property accesses to print (e.g., ξ μῆκος λέγε)
            if !asm_stmt.property_accesses.is_empty() {
                let mut args = Vec::new();
                for (owner, method) in &asm_stmt.property_accesses {
                    let receiver = AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(owner.clone()),
                        glossa_type: scope.lookup(owner).cloned().unwrap_or(GlossaType::Unknown),
                    };
                    args.push(AnalyzedExpr {
                        expr: AnalyzedExprKind::MethodCall {
                            receiver: Box::new(receiver),
                            method: method.clone(),
                            args: vec![],
                        },
                        glossa_type: GlossaType::Number, // len() returns usize
                    });
                }
                return Ok((StatementKind::Print, args));
            }

            // Check if we have index accesses to print
            if !asm_stmt.index_accesses.is_empty() {
                let mut args = Vec::new();
                for (array_expr, index_expr) in &asm_stmt.index_accesses {
                    let array_analyzed = convert_expr_to_analyzed(array_expr);
                    let index_analyzed = convert_expr_to_analyzed(index_expr);
                    args.push(AnalyzedExpr {
                        expr: AnalyzedExprKind::IndexAccess {
                            array: Box::new(array_analyzed),
                            index: Box::new(index_analyzed),
                        },
                        glossa_type: GlossaType::Unknown,
                    });
                }
                return Ok((StatementKind::Print, args));
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
    // If we have property accesses, use the first one
    if let Some((owner, method)) = asm_stmt.property_accesses.first() {
        let receiver = AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(owner.clone()),
            glossa_type: GlossaType::Unknown,
        };
        return (
            AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(receiver),
                    method: method.clone(),
                    args: vec![],
                },
                glossa_type: GlossaType::Number,
            },
            GlossaType::Number,
        );
    }

    // If we have index accesses, use the first one
    if let Some((array_expr, index_expr)) = asm_stmt.index_accesses.first() {
        let array_analyzed = convert_expr_to_analyzed(array_expr);
        let index_analyzed = convert_expr_to_analyzed(index_expr);
        return (
            AnalyzedExpr {
                expr: AnalyzedExprKind::IndexAccess {
                    array: Box::new(array_analyzed),
                    index: Box::new(index_analyzed),
                },
                glossa_type: GlossaType::Unknown, // Element type is unknown without inference
            },
            GlossaType::Unknown,
        );
    }

    // If we have arrays, use the first array
    if let Some(array_elements) = asm_stmt.arrays.first() {
        let analyzed_elements = convert_array_elements(array_elements);
        let element_type = analyzed_elements.first()
            .map(|e| e.glossa_type.clone())
            .unwrap_or(GlossaType::Unknown);
        return (
            AnalyzedExpr {
                expr: AnalyzedExprKind::ArrayLiteral(analyzed_elements),
                glossa_type: GlossaType::List(Box::new(element_type)),
            },
            GlossaType::List(Box::new(GlossaType::Unknown)),
        );
    }

    // If we have operators, build a binary expression
    if !asm_stmt.operators.is_empty() {
        // Check if we can build from literals alone (2+ literals)
        if asm_stmt.literals.len() >= 2 {
            let exprs = build_expressions_from_literals_and_ops(
                &asm_stmt.literals,
                &asm_stmt.operators,
            );
            if let Some(expr) = exprs.into_iter().next() {
                let ty = expr.glossa_type.clone();
                return (expr, ty);
            }
        }

        // Or check if we can combine object + literal with operator
        if let Some(ref obj) = asm_stmt.object {
            if !asm_stmt.literals.is_empty() {
                // Build: object op literal
                let left = AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                    glossa_type: GlossaType::Unknown, // Will be inferred
                };
                let right = literal_to_analyzed_expr(&asm_stmt.literals[0]);
                let op = asm_stmt.operators[0];
                let bin_expr = build_binary_expr(left, op, right);
                let ty = bin_expr.glossa_type.clone();
                return (bin_expr, ty);
            }
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

/// Convert an Expr to an AnalyzedExpr
fn convert_expr_to_analyzed(expr: &Expr) -> AnalyzedExpr {
    match expr {
        Expr::StringLiteral(s) => AnalyzedExpr {
            expr: AnalyzedExprKind::StringLiteral(s.clone()),
            glossa_type: GlossaType::String,
        },
        Expr::NumberLiteral(n) => AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(*n),
            glossa_type: GlossaType::Number,
        },
        Expr::BooleanLiteral(b) => AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(*b),
            glossa_type: GlossaType::Boolean,
        },
        Expr::Word(w) => {
            // Check if it's a numeral
            if let Some(val) = crate::morphology::lexicon::numeral_value(&w.normalized) {
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(val),
                    glossa_type: GlossaType::Number,
                }
            } else {
                // Variable reference
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(w.normalized.clone()),
                    glossa_type: GlossaType::Unknown,
                }
            }
        }
        Expr::ArrayLiteral(elements) => {
            let analyzed_elements = convert_array_elements(elements);
            let element_type = analyzed_elements.first()
                .map(|e| e.glossa_type.clone())
                .unwrap_or(GlossaType::Unknown);
            AnalyzedExpr {
                expr: AnalyzedExprKind::ArrayLiteral(analyzed_elements),
                glossa_type: GlossaType::List(Box::new(element_type)),
            }
        }
        Expr::IndexAccess { array, index } => {
            AnalyzedExpr {
                expr: AnalyzedExprKind::IndexAccess {
                    array: Box::new(convert_expr_to_analyzed(array)),
                    index: Box::new(convert_expr_to_analyzed(index)),
                },
                glossa_type: GlossaType::Unknown,
            }
        }
        _ => AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(0),
            glossa_type: GlossaType::Number,
        },
    }
}

/// Convert array elements from Expr to AnalyzedExpr
fn convert_array_elements(elements: &[Expr]) -> Vec<AnalyzedExpr> {
    elements.iter().map(|e| match e {
        Expr::StringLiteral(s) => AnalyzedExpr {
            expr: AnalyzedExprKind::StringLiteral(s.clone()),
            glossa_type: GlossaType::String,
        },
        Expr::NumberLiteral(n) => AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(*n),
            glossa_type: GlossaType::Number,
        },
        Expr::BooleanLiteral(b) => AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(*b),
            glossa_type: GlossaType::Boolean,
        },
        Expr::Word(w) => {
            // Check if it's a numeral
            if let Some(val) = crate::morphology::lexicon::numeral_value(&w.normalized) {
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(val),
                    glossa_type: GlossaType::Number,
                }
            } else {
                // Variable reference
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(w.normalized.clone()),
                    glossa_type: GlossaType::Unknown,
                }
            }
        }
        _ => AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(0),
            glossa_type: GlossaType::Number,
        },
    }).collect()
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
    /// Return: δός value
    Return { value: Option<Box<AnalyzedExpr>> },
    /// Function definition: name ὁρίζειν params· body
    FunctionDef {
        name: String,
        params: Vec<(String, Option<GlossaType>)>,
        body: Vec<AnalyzedStatement>,
        return_type: Option<GlossaType>,
    },
    /// Type definition: εἶδος name ὁρίζειν { fields }
    TypeDefinition {
        name: String,
        fields: Vec<(String, GlossaType)>,
    },
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
    /// Array literal [1, 2, 3]
    ArrayLiteral(Vec<AnalyzedExpr>),
    /// Index access array[index]
    IndexAccess {
        array: Box<AnalyzedExpr>,
        index: Box<AnalyzedExpr>,
    },
    /// Function call to user-defined function
    FunctionCall {
        func: String,
        args: Vec<AnalyzedExpr>,
    },
    /// Method call receiver.method(args)
    MethodCall {
        receiver: Box<AnalyzedExpr>,
        method: String,
        args: Vec<AnalyzedExpr>,
    },
    /// Struct instantiation Type { field: value, ... }
    StructInstantiation {
        type_name: String,
        args: Vec<AnalyzedExpr>,
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
        let (kind, analyzed_exprs) = self.analyze_expression_list(first_expr, stmt.is_query())?;

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
