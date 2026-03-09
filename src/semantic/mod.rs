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

pub mod analyzer;
#[cfg(test)]
mod assembler_tests;
pub mod assembly;
#[cfg(test)]
mod classification_tests;
pub(crate) mod control_flow;
pub(crate) mod conversion;
#[cfg(test)]
mod conversion_tests;
pub(crate) mod declarations;
pub(crate) mod expressions;
pub(crate) mod model;
pub(crate) mod patterns;
#[cfg(test)]
mod property_access_tests;
#[cfg(test)]
mod recursion_safety_tests;
mod resolver;
#[cfg(test)]
mod sentry_conversion_tests;

mod types;
pub(crate) mod validation;

pub use crate::morphology::{DisambiguationContext, analyze_article, disambiguate, resolve_best};
pub use analyzer::{AnalyzedProgram, analyze_program};
pub use assembly::Assembler;
pub use assembly::{
    AssembledStatement, AssemblyError, Constituent, Literal, ParticipleConstituent, VerbConstituent,
};
pub use model::*;
pub use resolver::*;

pub use types::*;

use self::expressions::feed_expr_to_assembler_with_context;
use crate::ast::Statement;
use crate::errors::GlossaError;

/// Analyze a single statement using the slot-based assembler
pub fn assemble_statement(stmt: &Statement) -> Result<AssembledStatement, GlossaError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_analyze_hello() {
        let ast = parse("«χαῖρε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        assert_eq!(analyzed.statements.len(), 1);
        assert!(matches!(
            analyzed.statements[0],
            AnalyzedStatement::Print(_)
        ));
    }

    #[test]
    fn test_analyze_binding() {
        let ast = parse("ξ πέντε ἔστω.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        assert!(matches!(
            &analyzed.statements[0],
            AnalyzedStatement::Binding { name, .. } if name == "ξ"
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
        assert!(matches!(
            analyzed.statements[1],
            AnalyzedStatement::Print(_)
        ));
    }

    #[test]
    fn test_analyze_string_literal() {
        let ast = parse("«χαῖρε κόσμε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        if let AnalyzedStatement::Print(exprs) = &analyzed.statements[0] {
            assert_eq!(exprs[0].glossa_type, GlossaType::String);
        } else {
            panic!("Expected Print statement");
        }
    }

    #[test]
    fn test_analyze_number_literal() {
        let ast = parse("42 λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        if let AnalyzedStatement::Print(exprs) = &analyzed.statements[0] {
            assert_eq!(exprs[0].glossa_type, GlossaType::Number);
        } else {
            panic!("Expected Print statement");
        }
    }
}
