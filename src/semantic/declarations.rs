//! Declaration analysis (functions, types, traits)

use super::{
    AnalyzedImplMethod, AnalyzedStatement, AnalyzedTraitMethod, GlossaType, Scope, StatementKind,
    analyze_single_statement_with_assembler, convert_assembled_to_analyzed,
};
use crate::ast::{Expr, Statement};
use crate::errors::GlossaError;
use crate::morphology;
use crate::text::normalize_greek;
use smol_str::SmolStr;
// Circular dependencies handled by crate structure
use super::control_flow::analyze_control_flow;
use super::patterns::{try_parse_struct_instantiation, try_parse_trait_method_call};

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
                    // Properly analyze statements in the body
                    for body_stmt in body_stmts {
                        if let Some(control_flow) = analyze_control_flow(body_stmt, &mut scope)? {
                            analyzed_body.push(control_flow);
                        } else if let Some(struct_inst) =
                            try_parse_struct_instantiation(body_stmt, &mut scope)?
                        {
                            analyzed_body.push(struct_inst);
                        } else if let Some(method_call) =
                            try_parse_trait_method_call(body_stmt, &mut scope)?
                        {
                            analyzed_body.push(method_call);
                        } else {
                            let assembled = analyze_single_statement_with_assembler(body_stmt)?;
                            let analyzed = convert_assembled_to_analyzed(&assembled, &mut scope)?;
                            analyzed_body.push(analyzed);
                        }
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

            analyzed_methods.push(AnalyzedTraitMethod {
                name: method_name,
                params,
                is_default: true,
                body,
                return_type,
            });
        } else {
            analyzed_methods.push(AnalyzedTraitMethod {
                name: method_name,
                params,
                is_default: false,
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

    Ok(AnalyzedStatement {
        kind: StatementKind::TraitDefinition {
            name: trait_name,
            methods: analyzed_methods,
        },
        expressions: vec![],
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

            // Analyze the method body
            for body_stmt in &method.body {
                // Try control flow first
                if let Some(control_flow) = analyze_control_flow(body_stmt, &mut scope)? {
                    analyzed_body.push(control_flow);
                }
                // Try struct instantiation pattern
                else if let Some(struct_inst) =
                    try_parse_struct_instantiation(body_stmt, &mut scope)?
                {
                    analyzed_body.push(struct_inst);
                }
                // Try trait method call pattern
                else if let Some(method_call) =
                    try_parse_trait_method_call(body_stmt, &mut scope)?
                {
                    analyzed_body.push(method_call);
                } else {
                    // Use assembler for regular statements
                    let assembled = analyze_single_statement_with_assembler(body_stmt)?;
                    let analyzed = convert_assembled_to_analyzed(&assembled, &mut scope)?;
                    analyzed_body.push(analyzed);
                }
            }
        }

        let return_type = infer_return_type_from_body(&analyzed_body);

        implemented_method_names.push(method_name.clone());
        analyzed_methods.push(AnalyzedImplMethod {
            name: method_name,
            params,
            body: analyzed_body,
            return_type,
        });
    }

    // Validate: all required methods must be implemented
    for method in &trait_def.methods {
        if !method.is_default && !implemented_method_names.contains(&method.name) {
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

    Ok(AnalyzedStatement {
        kind: StatementKind::TraitImplementation {
            trait_name,
            type_name,
            methods: analyzed_methods,
        },
        expressions: vec![],
    })
}

/// Map a genitive type name to a GlossaType
pub fn map_genitive_to_type(genitive_name: &str, scope: &Scope) -> GlossaType {
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

/// Parse a function definition: name ὁρίζειν [params]· body
pub fn parse_function_definition(
    stmt: &Statement,
    _scope: &mut Scope,
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
            is_propagate: false,
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
pub fn infer_return_type_from_body(body: &[AnalyzedStatement]) -> Option<GlossaType> {
    for stmt in body {
        if let StatementKind::Return { value } = &stmt.kind
            && let Some(return_expr) = value
        {
            return Some(return_expr.glossa_type.clone());
        }
    }
    None
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
            // Handle control flow
            if let Some(control_flow) = analyze_control_flow(body_stmt, &mut test_scope)? {
                analyzed_body.push(control_flow);
            }
            // Handle struct instantiation
            else if let Some(struct_inst) = try_parse_struct_instantiation(body_stmt, &mut test_scope)? {
                analyzed_body.push(struct_inst);
            }
            // Handle trait method calls
            else if let Some(method_call) = try_parse_trait_method_call(body_stmt, &mut test_scope)? {
                analyzed_body.push(method_call);
            }
            // Regular statement analysis
            else {
                let assembled = analyze_single_statement_with_assembler(body_stmt)?;
                let analyzed = convert_assembled_to_analyzed(&assembled, &mut test_scope)?;
                analyzed_body.push(analyzed);
            }
        }
    }

    Ok(AnalyzedStatement {
        kind: StatementKind::TestDeclaration {
            name: test_name,
            body: analyzed_body,
        },
        expressions: vec![],
    })
}
