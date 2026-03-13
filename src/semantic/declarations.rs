//! Declaration analysis for ΓΛΩΣΣΑ
//!
//! This module handles the analysis of declaration statements:
//! - Type definitions (εἶδος)
//! - Trait definitions (χαρακτήρ)
//! - Trait implementations (ὁρίζειν for traits)
//! - Function definitions (ὁρίζειν for functions)
//! - Test declarations (δοκιμή)

use super::{AnalyzedMethod, AnalyzedStatement, GlossaType, Scope, StatementAnalyzer};
use crate::ast::{Expr, Statement};
use crate::errors::GlossaError;
use crate::morphology::{self};
use smol_str::SmolStr;

/// Analyze a type definition statement
///
/// This processes `εἶδος` (struct) definitions. It registers the new type in the `Scope`
/// and validates fields for recursive dependencies.
///
/// # Greek Syntax
///
/// ```glossa
/// // "Form User to define..."
/// εἶδος Χρήστης ὁρίζειν {
///     ὄνομα ὀνόματος.
///     ἡλικία ἀριθμοῦ.
/// }.
/// ```
///
/// # Side Effects
///
/// * Updates `scope` with the new type definition.
/// * Allows recursive types via `Box`, `Vec`, etc., but prevents direct infinite recursion.
pub fn analyze_type_definition(
    type_def: &crate::ast::TypeDef,
    scope: &mut Scope,
    _analyzer: &mut impl StatementAnalyzer,
) -> Result<AnalyzedStatement, GlossaError> {
    // Extract type name
    let type_name = type_def.name.normalized.clone();

    // Define placeholder to allow recursive reference
    scope.define_type(
        type_name.clone(),
        GlossaType::Struct {
            name: type_name.clone(),
            gender: crate::morphology::Gender::Neuter,
            fields: vec![],
        },
    );

    // Analyze fields
    let mut fields = Vec::with_capacity(type_def.fields.len());
    for field in &type_def.fields {
        let field_name = field.name.normalized.clone();
        let type_name_gen = &field.type_name.normalized;

        // Map genitive type names to GlossaType
        let field_type = resolve_type_name(type_name_gen, scope)?;

        // Check for infinite recursion
        if check_recursive_type(&type_name, &field_type) {
            return Err(GlossaError::semantic(format!(
                "Recursive type detected: field '{}' uses type '{}' directly. Use a collection (List) or Box for indirection.",
                field_name, type_name
            )));
        }

        fields.push((field_name, field_type));
    }

    // Create the struct type
    let struct_type = GlossaType::Struct {
        name: type_name.clone(),
        gender: crate::morphology::Gender::Neuter, // Default for now
        fields: fields.clone(),
    };

    // Store the type in scope (update placeholder)
    scope.define_type(type_name.clone(), struct_type);

    Ok(AnalyzedStatement::TypeDefinition {
        name: type_name,
        fields,
    })
}

/// Check if a type contains the target type recursively without indirection
fn check_recursive_type(target_name: &str, ty: &GlossaType) -> bool {
    match ty {
        GlossaType::Struct { name, .. } => name == target_name,
        GlossaType::Option(inner) => check_recursive_type(target_name, inner),
        GlossaType::Result(ok, err) => {
            check_recursive_type(target_name, ok) || check_recursive_type(target_name, err)
        }
        // List, Set, Map break recursion
        _ => false,
    }
}

/// Analyze a trait definition statement
///
/// This processes `χαρακτήρ` (trait) definitions. It registers the trait signature
/// in the `Scope`.
///
/// # Greek Syntax
///
/// ```glossa
/// // "Character Printable to define..."
/// χαρακτήρ Εκτυπώσιμος ὁρίζειν {
///     τύπωσις(εαυτός). // Method signature
/// }.
/// ```
pub fn analyze_trait_definition(
    trait_def: &crate::ast::TraitDef,
    scope: &mut Scope,
    analyzer: &mut impl StatementAnalyzer,
) -> Result<AnalyzedStatement, GlossaError> {
    // Extract trait name
    let trait_name = trait_def.name.normalized.clone();

    // Check for duplicate trait definition
    if scope.lookup_trait(&trait_name).is_some() {
        return Err(GlossaError::semantic(format!(
            "Trait {} is already defined",
            trait_name
        )));
    }

    // Analyze methods
    let mut analyzed_methods = Vec::with_capacity(trait_def.methods.len());

    for method in &trait_def.methods {
        let method_name = method.name.normalized.clone();

        // Analyze method parameters
        let mut params = Vec::with_capacity(method.params.len());
        for param in &method.params {
            let param_name = param.name.normalized.clone();
            // For now, trait method parameters don't have explicit types
            // They'll be inferred based on the impl
            params.push((param_name, GlossaType::Unknown));
        }

        if method.is_default {
            // Analyze default method body
            let body = if let Some(body_stmts) = &method.body {
                let mut analyzed_body = Vec::with_capacity(body_stmts.len());
                // Create a child scope for the method
                {
                    let mut scope = scope.enter_scope();
                    // Add method parameters to scope (including self)
                    for (param_name, param_type) in &params {
                        scope.define(param_name.clone(), param_type.clone());
                    }
                    // Properly analyze statements in the body using unified helper
                    for body_stmt in body_stmts {
                        analyzed_body.extend(analyzer.analyze_statement(body_stmt, &mut scope)?);
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
///
/// This processes implementations of traits for specific types.
///
/// # Greek Syntax
///
/// ```glossa
/// // "Type User to the Printable to fall into..." (impl Printable for User)
/// εἶδος Χρήστης τῷ Εκτυπώσιμος ἐμπίπτειν {
///     τύπωσις(εαυτός) { ... }
/// }.
/// ```
pub fn analyze_trait_impl(
    trait_impl: &crate::ast::TraitImplDef,
    scope: &mut Scope,
    analyzer: &mut impl StatementAnalyzer,
) -> Result<AnalyzedStatement, GlossaError> {
    // Extract type and trait names
    let type_name = trait_impl.type_name.normalized.clone();
    let trait_name = trait_impl.trait_name.normalized.clone();

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
    let mut implemented_method_names = Vec::with_capacity(trait_impl.methods.len());
    let mut analyzed_methods = Vec::with_capacity(trait_impl.methods.len());

    for method in &trait_impl.methods {
        let method_name = method.name.normalized.clone();

        // Analyze method parameters
        let mut params = Vec::with_capacity(method.params.len());
        for param in &method.params {
            let param_name = param.name.normalized.clone();
            // For now, trait method parameters don't have explicit types
            params.push((param_name, GlossaType::Unknown));
        }

        // Create a child scope for the method body with self bound
        let mut analyzed_body = Vec::with_capacity(method.body.len());
        {
            let mut scope = scope.enter_scope();
            scope.define("self".to_string(), struct_type.clone());

            // Also bind parameters
            for (param_name, param_type) in &params {
                scope.define(param_name.clone(), param_type.clone());
            }

            // Analyze the method body using unified helper
            for body_stmt in &method.body {
                analyzed_body.extend(analyzer.analyze_statement(body_stmt, &mut scope)?);
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

/// Map a genitive type name to a GlossaType
pub fn resolve_type_name(name: &str, scope: &Scope) -> Result<GlossaType, GlossaError> {
    match name {
        // ἀριθμοῦ (genitive of ἀριθμός) → Number
        "αριθμου" => Ok(GlossaType::Number),
        // ὀνόματος (genitive of ὄνομα) → String
        "ονοματος" => Ok(GlossaType::String),
        // λιστης (genitive of λίστα) → List
        "λιστης" => Ok(GlossaType::List(Box::new(GlossaType::Unknown))),
        _ => {
            // Check for user-defined types
            // Strip genitive ending and look up the nominative form
            // For now, just try the name as-is
            if let Some(ty) = scope.lookup_type(name) {
                Ok(ty.clone())
            } else {
                Err(GlossaError::semantic(format!(
                    "Undefined type: '{}'. Ensure the type is defined before use.",
                    name
                )))
            }
        }
    }
}

/// Parse a function definition: name ὁρίζειν \[params\]· body
///
/// This handles the `ὁρίζειν` verb when used to define standalone functions.
///
/// # Greek Syntax
///
/// ```glossa
/// // "add to define (to the x of number, to the y of number) · ..."
/// προσθέτειν ὁρίζειν τῷ ξ ἀριθμοῦ τῷ ψ ἀριθμοῦ ·
///     ξ ψ ἄθροισμα δός.
/// ```
///
/// # Structure
///
/// 1. **Header**: Name + `ὁρίζειν` + Parameters (Dative + Genitive type).
/// 2. **Body**: Expressions separated by middle dot (·).
pub fn parse_function_definition(
    stmt: &Statement,
    scope: &mut Scope,
    analyzer: &mut impl StatementAnalyzer,
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
    // We can pre-allocate since we know exactly how many expressions form the body
    let mut body_statements = Vec::with_capacity(clause.expressions.len().saturating_sub(1));
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
            body_statements.extend(analyzer.analyze_statement(&clause_stmt, &mut function_scope)?);
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
        let normalized = &word.normalized;

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
            function_name = Some(normalized.clone());
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
        let normalized = &word.normalized;
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
                let param_name = param_word.normalized.clone();

                // Check if word after param is a genitive (type annotation)
                let mut param_type = None;
                if i + 2 < words.len() {
                    let next_word = &words[i + 2];
                    let next_analysis = morphology::analyze(&next_word.original);
                    if next_analysis.case == Some(morphology::Case::Genitive) {
                        // Map genitive type to GlossaType
                        let type_name = &next_word.normalized;
                        // If it looks like a type annotation (genitive), it MUST be a valid type
                        param_type = Some(resolve_type_name(type_name, scope)?);
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
    collect_words_from_expr_into(expr, &mut words);
    words
}

fn collect_words_from_expr_into<'a>(expr: &'a Expr, words: &mut Vec<&'a crate::ast::Word>) {
    match expr {
        Expr::Word(word) => words.push(word),
        Expr::Phrase(terms) => {
            for term in terms {
                collect_words_from_expr_into(term, words);
            }
        }
        _ => {}
    }
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

/// Analyze a test declaration statement
pub fn analyze_test_declaration(
    test_decl: &crate::ast::TestDecl,
    scope: &mut Scope,
    analyzer: &mut impl StatementAnalyzer,
) -> Result<AnalyzedStatement, GlossaError> {
    let test_name = test_decl.name.clone();

    // Analyze the test body statements
    let mut analyzed_body = Vec::with_capacity(test_decl.body.len());

    // Create a child scope for the test
    {
        let mut test_scope = scope.enter_scope();

        for body_stmt in &test_decl.body {
            analyzed_body.extend(analyzer.analyze_statement(body_stmt, &mut test_scope)?);
        }
    }

    Ok(AnalyzedStatement::TestDeclaration {
        name: test_name,
        body: analyzed_body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_recursive_type() {
        let target = "Target";

        // Direct recursion: Target == Target
        let direct = GlossaType::Struct {
            name: "Target".into(),
            gender: crate::morphology::Gender::Neuter,
            fields: vec![],
        };
        assert!(check_recursive_type(target, &direct));

        // Non-recursive struct
        let other = GlossaType::Struct {
            name: "Other".into(),
            gender: crate::morphology::Gender::Neuter,
            fields: vec![],
        };
        assert!(!check_recursive_type(target, &other));

        // Option recursion: Option<Target>
        let opt_rec = GlossaType::Option(Box::new(direct.clone()));
        assert!(check_recursive_type(target, &opt_rec));

        // Option non-recursion: Option<Other>
        let opt_non_rec = GlossaType::Option(Box::new(other.clone()));
        assert!(!check_recursive_type(target, &opt_non_rec));

        // Result recursion (Ok): Result<Target, Other>
        let res_ok_rec = GlossaType::Result(Box::new(direct.clone()), Box::new(other.clone()));
        assert!(check_recursive_type(target, &res_ok_rec));

        // Result recursion (Err): Result<Other, Target>
        let res_err_rec = GlossaType::Result(Box::new(other.clone()), Box::new(direct.clone()));
        assert!(check_recursive_type(target, &res_err_rec));

        // Result non-recursion: Result<Other, Other>
        let res_non_rec = GlossaType::Result(Box::new(other.clone()), Box::new(other.clone()));
        assert!(!check_recursive_type(target, &res_non_rec));

        // List breaks recursion: List<Target> -> Safe (Vec is heap allocated)
        let list_rec = GlossaType::List(Box::new(direct.clone()));
        assert!(!check_recursive_type(target, &list_rec));

        // Set breaks recursion
        let set_rec = GlossaType::Set(Box::new(direct.clone()));
        assert!(!check_recursive_type(target, &set_rec));

        // Map breaks recursion
        let map_rec = GlossaType::Map(Box::new(direct.clone()), Box::new(other.clone()));
        assert!(!check_recursive_type(target, &map_rec));
    }
}
