//! Statement analysis (control flow, declarations, patterns)
//!
//! This module consolidates logic for parsing various statement types:
//! - Control flow (if, while, for, match)
//! - Declarations (functions, types, traits)
//! - High-level patterns (struct instantiation, trait methods)

use super::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedMethod, AnalyzedStatement, GlossaType, Scope,
    analyze_statement, assemble_statement, convert_assembled_to_analyzed,
};
use crate::ast::{Clause, Expr, Statement};
use crate::errors::GlossaError;
use crate::morphology::{self, lexicon};
use crate::text::normalize_greek;
use smol_str::SmolStr;

use super::expressions::{contains_function_definition_verb, get_first_word};

/// Check if a statement is a control flow construct or declaration and analyze it
/// Returns Some(AnalyzedStatement) if recognized, None otherwise
pub fn analyze_control_flow(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    // Check for function definition (contains ὁρίζειν)
    if contains_function_definition_verb(stmt) {
        return parse_function_definition(stmt, scope);
    }

    // Get the first word to check for control flow particles
    // If this fails (e.g., statement starts with literal), it might still be a pattern
    let first_word_res = get_first_word(stmt);

    if let Ok(first_word) = first_word_res {
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
            return Ok(Some(AnalyzedStatement::Break));
        }

        // Continue: συνέχιζε
        if lexicon::is_continue_verb(&normalized) {
            return Ok(Some(AnalyzedStatement::Continue));
        }
    }

    // Check for struct instantiation pattern
    // Pattern: [variable] νέον [TypeName] [arg1] ... ἔστω
    // This doesn't necessarily start with a specific keyword, so we check heuristics
    if let Some(struct_inst) = try_parse_struct_instantiation(stmt, scope)? {
        return Ok(Some(struct_inst));
    }

    // Check for trait method call pattern
    // Pattern: [method_name] [receiver]
    if let Some(method_call) = try_parse_trait_method_call(stmt, scope)? {
        return Ok(Some(method_call));
    }

    // Not a control flow construct or special pattern
    Ok(None)
}

/// Analyze a type definition statement
pub fn analyze_type_definition(
    type_def: &crate::ast::TypeDef,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    // Extract type name
    let type_name = normalize_greek(&type_def.name.original);

    // Analyze fields
    let mut fields = Vec::new();
    for field in &type_def.fields {
        let field_name = normalize_greek(&field.name.original);
        let type_name_gen = normalize_greek(&field.type_name.original);

        // Map genitive type names to GlossaType
        let field_type = super::types::resolve_type_name(&type_name_gen, scope);
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

    Ok(AnalyzedStatement::TypeDefinition {
        name: type_name,
        fields,
    })
}

/// Analyze a trait definition statement
pub fn analyze_trait_definition(
    trait_def: &crate::ast::TraitDef,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    // Extract trait name
    let trait_name = normalize_greek(&trait_def.name.original);

    // Check for duplicate trait definition
    if scope.lookup_trait(&trait_name).is_some() {
        return Err(GlossaError::semantic(format!(
            "Trait {} is already defined",
            trait_name
        )));
    }

    // Analyze methods
    let mut analyzed_methods = Vec::new();

    for method in &trait_def.methods {
        let method_name = normalize_greek(&method.name.original);

        // Analyze method parameters
        let mut params = Vec::new();
        for param in &method.params {
            let param_name = normalize_greek(&param.name.original);
            // For now, trait method parameters don't have explicit types
            // They'll be inferred based on the impl
            params.push((param_name, GlossaType::Unknown));
        }

        if method.is_default {
            // Analyze default method body
            let body = if let Some(body_stmts) = &method.body {
                let mut analyzed_body = Vec::new();
                // Create a child scope for the method
                {
                    let mut scope = scope.enter_scope();
                    // Add method parameters to scope (including self)
                    for (param_name, param_type) in &params {
                        scope.define(param_name.clone(), param_type.clone());
                    }
                    // Properly analyze statements in the body using unified helper
                    for body_stmt in body_stmts {
                        analyzed_body.extend(analyze_statement(body_stmt, &mut scope)?);
                    }
                }
                Some(analyzed_body)
            } else {
                Some(vec![])
            };

            let return_type = if let Some(body) = &body {
                infer_return_type_from_body(body)
            } else {
                None
            };

            analyzed_methods.push(AnalyzedMethod {
                name: method_name,
                params,
                body,
                return_type,
            });
        } else {
            analyzed_methods.push(AnalyzedMethod {
                name: method_name,
                params,
                body: None,
                return_type: None,
            });
        }
    }

    // Create the trait definition
    let trait_def_semantic = crate::semantic::model::TraitDef {
        name: trait_name.clone(),
        methods: analyzed_methods.clone(),
    };

    // Store the trait in scope
    scope.define_trait(trait_name.clone(), trait_def_semantic);

    Ok(AnalyzedStatement::TraitDefinition {
        name: trait_name,
        methods: analyzed_methods,
    })
}

/// Analyze a trait implementation statement
pub fn analyze_trait_impl(
    trait_impl: &crate::ast::TraitImplDef,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    // Extract type and trait names
    let type_name = normalize_greek(&trait_impl.type_name.original);
    let trait_name = normalize_greek(&trait_impl.trait_name.original);

    // Validate: trait must exist
    let trait_def = scope
        .lookup_trait(&trait_name)
        .cloned()
        .ok_or_else(|| GlossaError::semantic(format!("Trait {} is not defined", trait_name)))?;

    // Validate: type must exist
    let struct_type = scope
        .lookup_type(&type_name)
        .cloned()
        .ok_or_else(|| GlossaError::semantic(format!("Type {} is not defined", type_name)))?;

    // Collect implemented methods
    let mut implemented_method_names = Vec::new();
    let mut analyzed_methods = Vec::new();

    for method in &trait_impl.methods {
        let method_name = normalize_greek(&method.name.original);

        // Analyze method parameters
        let mut params = Vec::new();
        for param in &method.params {
            let param_name = normalize_greek(&param.name.original);
            // For now, trait method parameters don't have explicit types
            params.push((param_name, GlossaType::Unknown));
        }

        // Create a child scope for the method body with self bound
        let mut analyzed_body = Vec::new();
        {
            let mut scope = scope.enter_scope();
            scope.define("self".to_string(), struct_type.clone());

            // Also bind parameters
            for (param_name, param_type) in &params {
                scope.define(param_name.clone(), param_type.clone());
            }

            // Analyze the method body using unified helper
            for body_stmt in &method.body {
                analyzed_body.extend(analyze_statement(body_stmt, &mut scope)?);
            }
        }

        let return_type = infer_return_type_from_body(&analyzed_body);

        implemented_method_names.push(method_name.clone());
        analyzed_methods.push(AnalyzedMethod {
            name: method_name,
            params,
            body: Some(analyzed_body),
            return_type,
        });
    }

    // Validate: all required methods must be implemented
    for method in &trait_def.methods {
        if method.body.is_none() && !implemented_method_names.contains(&method.name) {
            return Err(GlossaError::semantic(format!(
                "Type {} does not implement required method {} from trait {}",
                type_name, method.name, trait_name
            )));
        }
    }

    // Create the trait implementation
    let trait_impl_semantic = crate::semantic::model::TraitImpl {
        trait_name: trait_name.clone(),
        type_name: type_name.clone(),
    };

    // Register the trait impl in scope
    scope.register_trait_impl(trait_impl_semantic);

    Ok(AnalyzedStatement::TraitImplementation {
        trait_name,
        type_name,
        methods: analyzed_methods,
    })
}

/// Analyze a test declaration statement
pub fn analyze_test_declaration(
    test_decl: &crate::ast::TestDecl,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    let test_name = test_decl.name.clone();

    // Analyze the test body statements
    let mut analyzed_body = Vec::new();

    // Create a child scope for the test
    {
        let mut test_scope = scope.enter_scope();

        for body_stmt in &test_decl.body {
            analyzed_body.extend(analyze_statement(body_stmt, &mut test_scope)?);
        }
    }

    Ok(AnalyzedStatement::TestDeclaration {
        name: test_name,
        body: analyzed_body,
    })
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
    let mut arms = Vec::new();

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
fn parse_match_pattern(expr: &Expr, scope: &mut Scope) -> Result<AnalyzedExpr, GlossaError> {
    // Pattern is typically: value ᾖ
    if let Expr::Phrase(terms) = expr {
        if terms.is_empty() {
            return Err(GlossaError::semantic("Empty match pattern"));
        }

        // Get first word (the pattern value)
        if let Expr::Word(w) = &terms[0] {
            let normalized = normalize_greek(&w.original);

            // Check if it's ἄλλο (wildcard)
            if normalized == "αλλο" {
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

/// Parse a function definition: name ὁρίζειν [params]· body
pub fn parse_function_definition(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    // The middle dot (·) separates expressions within a clause
    // Structure: expr1 · expr2 where expr1 is "name ὁρίζειν [params]" and expr2 is the body

    // Get the first clause
    if stmt.clauses().is_empty() {
        return Err(GlossaError::semantic("Function definition cannot be empty"));
    }

    let clause = &stmt.clauses()[0];

    // We need at least 2 expressions: header and body
    if clause.expressions.len() < 2 {
        return Err(GlossaError::semantic(
            "Function definition needs header and body separated by middle dot (·)",
        ));
    }

    // Parse header (first expression contains name, ὁρίζειν, and parameters)
    let header_expr = &clause.expressions[0];
    let function_name = extract_function_name_from_expr(header_expr)?;

    // Extract parameters (dative words) from the header
    let params = extract_parameters_from_expr(header_expr, scope)?;

    // Use enter_scope() to create a child scope that inherits types/traits
    let mut body_statements = Vec::new();
    {
        let mut function_scope = scope.enter_scope();
        for (param_name, param_type) in &params {
            let glossa_type = param_type.clone().unwrap_or(GlossaType::Unknown);
            function_scope.define(param_name.clone(), glossa_type);
        }

        // Parse body (remaining expressions after middle dot)
        let body_exprs = &clause.expressions[1..];

        // Each expression in the body becomes a statement
        for expr in body_exprs {
            // Create a clause with this single expression
            let body_clause = crate::ast::Clause {
                expressions: vec![expr.clone()],
            };

            let clause_stmt = Statement::Regular {
                clauses: vec![body_clause],
                is_query: false,
                is_propagate: false,
            };

            // Analyze each body expression using unified helper
            body_statements.extend(analyze_statement(&clause_stmt, &mut function_scope)?);
        }
    }

    // Infer return type from return statements
    let return_type = infer_return_type_from_body(&body_statements);

    Ok(Some(AnalyzedStatement::FunctionDef {
        name: function_name,
        params,
        body: body_statements,
        return_type,
    }))
}

/// Extract the function name from a function definition header expression
fn extract_function_name_from_expr(expr: &Expr) -> Result<SmolStr, GlossaError> {
    // Collect all words and find the nominative word before ὁρίζειν
    let words = collect_words_from_expr(expr);

    let mut function_name = None;

    for word in &words {
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
fn extract_parameters_from_expr(
    expr: &Expr,
    scope: &Scope,
) -> Result<Vec<(SmolStr, Option<GlossaType>)>, GlossaError> {
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
                        param_type = Some(super::types::resolve_type_name(&type_name, scope));
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

/// Infer the return type from the function body by examining return statements
pub fn infer_return_type_from_body(body: &[AnalyzedStatement]) -> Option<GlossaType> {
    for stmt in body {
        if let AnalyzedStatement::Return { value } = stmt
            && let Some(return_expr) = value
        {
            return Some(return_expr.glossa_type.clone());
        }
    }
    None
}

/// Try to parse a trait method call: `method_name receiver`
/// Returns `Some(analyzed_statement)` if this is a trait method call, `None` otherwise.
pub fn try_parse_trait_method_call(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    // Only process Regular statements
    if let Statement::Regular { clauses, .. } = stmt {
        // Should have exactly one clause with one expression
        if clauses.len() != 1 || clauses[0].expressions.len() != 1 {
            return Ok(None);
        }

        // Should be a Phrase with exactly 2 words
        if let Expr::Phrase(terms) = &clauses[0].expressions[0] {
            if terms.len() != 2 {
                return Ok(None);
            }

            // Extract words
            if let (Expr::Word(method_word), Expr::Word(receiver_word)) = (&terms[0], &terms[1]) {
                let method_name = &method_word.normalized;
                let receiver_name = &receiver_word.normalized;

                // Check if receiver is a variable in scope
                if let Some(receiver_type) = scope.lookup(receiver_name)
                    && let GlossaType::Struct {
                        name: type_name, ..
                    } = receiver_type
                {
                    // Check if this type has a trait method with this name
                    if scope.has_trait_method(type_name, method_name) {
                        let receiver = AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(receiver_name.clone()),
                            glossa_type: receiver_type.clone(),
                        };

                        let method_call = AnalyzedExpr {
                            expr: AnalyzedExprKind::MethodCall {
                                receiver: Box::new(receiver),
                                method: method_name.clone(),
                                args: vec![],
                            },
                            glossa_type: GlossaType::Unit,
                        };

                        return Ok(Some(AnalyzedStatement::Expression(vec![method_call])));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Try to parse a struct instantiation
/// Returns `Some(analyzed_statement)` if this is a struct instantiation, `None` otherwise.
pub fn try_parse_struct_instantiation(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    // Only process Regular statements
    if let Statement::Regular { clauses, .. } = stmt {
        // Should have exactly one clause
        if clauses.len() != 1 {
            return Ok(None);
        }

        let clause = &clauses[0];
        if clause.expressions.len() != 1 {
            return Ok(None);
        }

        // Should be a Phrase with at least 4 terms
        if let Expr::Phrase(terms) = &clause.expressions[0] {
            if terms.len() < 4 {
                return Ok(None);
            }

            // Verify structural words (0, 1, 2, Last) are Words
            let var_word = if let Expr::Word(w) = &terms[0] {
                w
            } else {
                return Ok(None);
            };

            let adj_word = if let Expr::Word(w) = &terms[1] {
                w
            } else {
                return Ok(None);
            };

            let type_word = if let Expr::Word(w) = &terms[2] {
                w
            } else {
                return Ok(None);
            };

            let last_word = if let Expr::Word(w) = terms.last().unwrap() {
                w
            } else {
                return Ok(None);
            };

            // Check pattern: var_name νέον TypeName args... ἔστω
            // Last word should be ἔστω (binding verb)
            if !crate::morphology::lexicon::is_binding_verb(&last_word.normalized) {
                return Ok(None);
            }

            // Second word should be νέον (new) - check both normalized form and if it's "new" via morphology
            let normalized_adj = crate::text::normalize_greek(&adj_word.normalized);
            // Check if it's "new" - could be νέον, νεον, etc.
            if normalized_adj != "νεον" && normalized_adj != "νεος" {
                return Ok(None);
            }

            // Extract components
            let var_name = &var_word.normalized;
            let type_name = &type_word.normalized;

            // Check for built-in collection types first (HashSet, HashMap)
            if let Some((rust_type, glossa_type)) =
                super::types::detect_collection_type(type_name)
            {
                let collection_new = AnalyzedExpr {
                    expr: AnalyzedExprKind::CollectionNew {
                        collection_type: rust_type.to_string(),
                    },
                    glossa_type: glossa_type.clone(),
                };

                // Register variable in scope (collections are implicitly mutable for insert)
                scope.define_mut(var_name.clone(), glossa_type.clone());

                return Ok(Some(AnalyzedStatement::Binding {
                    name: var_name.clone(),
                    value: collection_new,
                    mutable: true,
                }));
            }

            // Check if type exists as a user-defined struct
            if let Some(struct_type) = scope.lookup_type(type_name).cloned() {
                // Extract fields from struct type
                let fields_info: Vec<(SmolStr, GlossaType)> =
                    if let GlossaType::Struct { fields, .. } = &struct_type {
                        fields.clone()
                    } else {
                        vec![]
                    };

                let field_names: Vec<SmolStr> =
                    fields_info.iter().map(|(n, _)| n.clone()).collect();

                // Collect constructor arguments (everything between type_name and ἔστω)
                let mut args = Vec::new();
                for (i, term) in terms[3..terms.len() - 1].iter().enumerate() {
                    let expected_type = if i < fields_info.len() {
                        &fields_info[i].1
                    } else {
                        &GlossaType::Unknown
                    };

                    let analyzed_arg = match term {
                        Expr::Word(word) => {
                            // Convert word to analyzed expression
                            if let Ok(num) = word.original.parse::<i64>() {
                                // Direct numeric literal like "5" stored as word
                                AnalyzedExpr {
                                    expr: AnalyzedExprKind::NumberLiteral(num),
                                    glossa_type: GlossaType::Number,
                                }
                            } else if let Some(num) =
                                crate::morphology::lexicon::numeral_value(&word.normalized)
                            {
                                // Greek numeral word like πέντε -> 5
                                AnalyzedExpr {
                                    expr: AnalyzedExprKind::NumberLiteral(num),
                                    glossa_type: GlossaType::Number,
                                }
                            } else {
                                // Variable reference
                                let var_type = scope
                                    .lookup(&word.normalized)
                                    .cloned()
                                    .unwrap_or(GlossaType::Unknown);
                                AnalyzedExpr {
                                    expr: AnalyzedExprKind::Variable(word.normalized.clone()),
                                    glossa_type: var_type,
                                }
                            }
                        }
                        Expr::StringLiteral(s) => {
                            let lit_expr = AnalyzedExpr {
                                expr: AnalyzedExprKind::StringLiteral(s.clone()),
                                glossa_type: GlossaType::String,
                            };

                            if matches!(expected_type, GlossaType::String) {
                                // Wrap in .to_string() for struct fields expecting String
                                AnalyzedExpr {
                                    expr: AnalyzedExprKind::MethodCall {
                                        receiver: Box::new(lit_expr),
                                        method: "to_string".into(),
                                        args: vec![],
                                    },
                                    glossa_type: GlossaType::String,
                                }
                            } else {
                                lit_expr
                            }
                        }
                        Expr::NumberLiteral(n) => AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(*n),
                            glossa_type: GlossaType::Number,
                        },
                        Expr::BooleanLiteral(b) => AnalyzedExpr {
                            expr: AnalyzedExprKind::BooleanLiteral(*b),
                            glossa_type: GlossaType::Boolean,
                        },
                        _ => return Ok(None), // Unsupported argument type
                    };
                    args.push(analyzed_arg);
                }

                // Build struct instantiation
                let struct_inst = AnalyzedExpr {
                    expr: AnalyzedExprKind::StructInstantiation {
                        type_name: type_name.clone(),
                        fields: field_names,
                        args,
                    },
                    glossa_type: struct_type.clone(),
                };

                // Register variable in scope with correct type
                scope.define(var_name.clone(), struct_type.clone());

                return Ok(Some(AnalyzedStatement::Binding {
                    name: var_name.clone(),
                    value: struct_inst,
                    mutable: false,
                }));
            } else {
                // If it looks like a struct instantiation (var νέον Type ... ἔστω)
                // but the type is unknown, return an error instead of falling back.
                return Err(GlossaError::undefined(type_name.to_string()));
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Clause, Expr, Statement, Word};
    use crate::semantic::{GlossaType, Scope};

    // Helper to create a dummy statement
    fn make_stmt(words: Vec<&str>) -> Statement {
        let expressions: Vec<Expr> = words
            .into_iter()
            .map(|w| Expr::Word(Word::new(w)))
            .collect();

        // If multiple words, wrap in Phrase as parser would, unless it's a single word
        let expr = if expressions.len() == 1 {
            expressions[0].clone()
        } else {
            Expr::Phrase(expressions)
        };

        Statement::Regular {
            clauses: vec![Clause {
                expressions: vec![expr],
            }],
            is_query: false,
            is_propagate: false,
        }
    }

    #[test]
    fn test_analyze_control_flow_empty_statement() {
        let mut scope = Scope::new();
        let stmt = Statement::Regular {
            clauses: vec![],
            is_query: false,
            is_propagate: false,
        };

        let result = analyze_control_flow(&stmt, &mut scope);
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_analyze_control_flow_break() {
        let mut scope = Scope::new();
        let stmt = make_stmt(vec!["παῦε"]);
        let result = analyze_control_flow(&stmt, &mut scope).unwrap();
        assert!(matches!(result, Some(AnalyzedStatement::Break)));
    }

    #[test]
    fn test_analyze_control_flow_continue() {
        let mut scope = Scope::new();
        let stmt = make_stmt(vec!["συνέχιζε"]);
        let result = analyze_control_flow(&stmt, &mut scope).unwrap();
        assert!(matches!(result, Some(AnalyzedStatement::Continue)));
    }

    #[test]
    fn test_struct_instantiation_not_enough_terms() {
        let mut scope = Scope::new();
        // "x new Type" (missing "let")
        let stmt = make_stmt(vec!["χ", "νέον", "Τύπος"]);
        let result = try_parse_struct_instantiation(&stmt, &mut scope).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_struct_instantiation_wrong_verb() {
        let mut scope = Scope::new();
        // "x new Type wrong"
        let stmt = make_stmt(vec!["χ", "νέον", "Τύπος", "λάθος"]);
        let result = try_parse_struct_instantiation(&stmt, &mut scope).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_struct_instantiation_not_new() {
        let mut scope = Scope::new();
        // "x old Type let"
        let stmt = make_stmt(vec!["χ", "παλαιόν", "Τύπος", "ἔστω"]);
        let result = try_parse_struct_instantiation(&stmt, &mut scope).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_struct_instantiation_unknown_type() {
        let mut scope = Scope::new();
        // "x new UnknownType let" - should ERROR, not return None
        let stmt = make_stmt(vec!["χ", "νέον", "Ἄγνωστον", "ἔστω"]);
        let result = try_parse_struct_instantiation(&stmt, &mut scope);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Ἄγνωστον"));
    }

    #[test]
    fn test_trait_method_call_wrong_length() {
        let mut scope = Scope::new();
        // "method receiver extra"
        let stmt = make_stmt(vec!["μέθοδος", "δέκτης", "ἄλλο"]);
        let result = try_parse_trait_method_call(&stmt, &mut scope).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_trait_method_call_unknown_receiver() {
        let mut scope = Scope::new();
        // "method unknown"
        let stmt = make_stmt(vec!["μέθοδος", "ἄγνωστος"]);
        let result = try_parse_trait_method_call(&stmt, &mut scope).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_trait_method_call_receiver_not_struct() {
        let mut scope = Scope::new();
        scope.define("χ", GlossaType::Number);
        // "method x" (x is Number, not Struct)
        let stmt = make_stmt(vec!["μέθοδος", "χ"]);
        let result = try_parse_trait_method_call(&stmt, &mut scope).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_trait_method_call_method_not_in_trait() {
        let mut scope = Scope::new();
        // Define a struct but no trait impl
        scope.define_type("Τύπος", GlossaType::Struct {
            name: "Τύπος".into(),
            gender: crate::morphology::Gender::Neuter,
            fields: vec![]
        });
        scope.define("τ", GlossaType::Struct {
            name: "Τύπος".into(),
            gender: crate::morphology::Gender::Neuter,
            fields: vec![]
        });

        // "method t"
        let stmt = make_stmt(vec!["μέθοδος", "τ"]);
        let result = try_parse_trait_method_call(&stmt, &mut scope).unwrap();
        assert!(result.is_none());
    }
}
