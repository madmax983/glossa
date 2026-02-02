//! Semantic analysis for ΓΛΩΣΣΑ
//!
//! This module implements the semantic analysis pipeline, which transforms the raw AST
//! into a typed, resolved representation ready for intermediate code generation.
//!
//! # The Analysis Pipeline
//!
//! 1. **Morphological Analysis**: Raw words are analyzed for case, gender, number, etc.
//!    (handled by the `morphology` module).
//! 2. **Slot-Based Assembly**: The [`Assembler`] routes these words into grammatical slots
//!    (Subject, Object, Verb) based on their case endings. This provides the
//!    language's signature free word order.
//! 3. **Pattern Recognition**: The assembled sentence is classified into a statement kind
//!    (Binding, Print, If, etc.) based on the verb and constituents.
//! 4. **Name Resolution**: Variables are looked up in the [`Scope`] to ensure they exist.
//! 5. **Type Inference**: Types are inferred from usage and lexical definitions.
//!
//! # The Assembler Approach
//!
//! Unlike traditional parsers that rely on fixed word positions (e.g., "verb follows subject"),
//! ΓΛΩΣΣΑ uses the `Assembler` to assemble sentences based on grammatical *roles*.
//!
//! ```text
//! "ὁ ἄνθρωπος τὸν λόγον λέγει"
//!      ↓           ↓       ↓
//! [Nominative] [Accusative] [Verb]
//!      ↓           ↓       ↓
//!   Subject      Object   Action
//! ```
//!
//! This allows for authentic Greek syntax where emphasis is conveyed through word order
//! without changing the semantic meaning.

pub mod assembler;
pub(crate) mod control_flow;
pub(crate) mod conversion;
pub(crate) mod declarations;
#[doc(hidden)]
pub mod expressions;
pub(crate) mod model;
pub(crate) mod patterns;
mod resolver;
mod types;

pub use crate::morphology::{DisambiguationContext, analyze_article, disambiguate, resolve_best};
pub use assembler::{
    AssembledStatement, Assembler, AssemblyError, Constituent, Literal, VerbConstituent,
};
pub use model::*;
pub use resolver::*;
pub use types::*;

use crate::ast::{Program, Statement};
use crate::errors::GlossaError;

use self::control_flow::analyze_control_flow;
use self::conversion::convert_assembled_to_analyzed;
use self::declarations::{analyze_trait_definition, analyze_trait_impl, analyze_type_definition};
use self::expressions::feed_expr_to_assembler_with_context;
use self::patterns::{try_parse_struct_instantiation, try_parse_trait_method_call};

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

        // Handle trait definitions
        if let Statement::TraitDefinition(trait_def) = stmt {
            analyzed_statements.push(analyze_trait_definition(trait_def, &mut scope)?);
            continue;
        }

        // Handle trait implementations
        if let Statement::TraitImpl(trait_impl) = stmt {
            analyzed_statements.push(analyze_trait_impl(trait_impl, &mut scope)?);
            continue;
        }

        // Check if this is a control flow construct
        if let Some(control_flow_stmt) = analyze_control_flow(stmt, &mut scope)? {
            // If it's a function definition, register it in the scope
            if let StatementKind::FunctionDef {
                name,
                params,
                return_type,
                ..
            } = &control_flow_stmt.kind
            {
                let param_types: Vec<GlossaType> = params
                    .iter()
                    .map(|(_, ty)| ty.clone().unwrap_or(GlossaType::Unknown))
                    .collect();
                scope.define_function(name.clone(), param_types, return_type.clone());
            }
            analyzed_statements.push(control_flow_stmt);
        } else {
            // Check for struct instantiation pattern BEFORE assembler
            // Pattern: var_name νέον TypeName args... ἔστω
            if let Some(struct_inst) = try_parse_struct_instantiation(stmt, &mut scope)? {
                analyzed_statements.push(struct_inst);
            }
            // Check for trait method call pattern BEFORE assembler
            // Pattern: method_name receiver
            else if let Some(method_call) = try_parse_trait_method_call(stmt, &mut scope)? {
                analyzed_statements.push(method_call);
            } else {
                // Use the assembler-based approach for regular statements
                let assembled = analyze_single_statement_with_assembler(stmt)?;
                let analyzed = convert_assembled_to_analyzed(&assembled, &mut scope)?;
                analyzed_statements.push(analyzed);
            }
        }
    }

    Ok(AnalyzedProgram {
        statements: analyzed_statements,
        scope,
    })
}

/// Analyze a single statement using the slot-based assembler
fn analyze_single_statement_with_assembler(
    stmt: &Statement,
) -> Result<AssembledStatement, GlossaError> {
    let mut asm = Assembler::new();
    asm.set_query(stmt.is_query());
    asm.set_propagate(stmt.is_propagate());

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
    Ok(asm.finalize()?)
}

/// Analyzed program with resolved names and types
#[derive(Debug, Clone)]
pub struct AnalyzedProgram {
    pub statements: Vec<AnalyzedStatement>,
    pub scope: Scope,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_analyze_hello() {
        let ast = parse("«χαῖρε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        assert_eq!(analyzed.statements.len(), 1);
        assert!(matches!(analyzed.statements[0].kind, StatementKind::Print));
    }

    #[test]
    fn test_analyze_binding() {
        let ast = parse("ξ πέντε ἔστω.").unwrap();
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
        let ast = parse("ξ πέντε ἔστω. ξ λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        assert_eq!(analyzed.statements.len(), 2);
        // Second statement should reference ξ with known type
        assert!(matches!(analyzed.statements[1].kind, StatementKind::Print));
    }

    #[test]
    fn test_analyze_string_literal() {
        let ast = parse("«χαῖρε κόσμε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        let first_expr = &analyzed.statements[0].expressions[0];
        assert_eq!(first_expr.glossa_type, GlossaType::String);
    }

    #[test]
    fn test_analyze_number_literal() {
        let ast = parse("42 λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        let first_expr = &analyzed.statements[0].expressions[0];
        assert_eq!(first_expr.glossa_type, GlossaType::Number);
    }
}
