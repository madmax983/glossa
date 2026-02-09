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

pub mod assembled;
pub mod assembler;
pub(crate) mod control_flow;
pub(crate) mod conversion;
pub(crate) mod declarations;
pub mod expressions;
pub(crate) mod model;
pub(crate) mod patterns;
mod resolver;
mod types;

pub use crate::morphology::{DisambiguationContext, analyze_article, disambiguate, resolve_best};
pub use assembled::{AssembledStatement, Constituent, Literal, VerbConstituent};
pub use assembler::{Assembler, AssemblyError};
pub use model::*;
pub use resolver::*;
pub use types::*;

use crate::ast::{Expr, Program, Statement};
use crate::errors::GlossaError;

use self::control_flow::analyze_control_flow;
use self::conversion::convert_assembled_to_analyzed;
use self::declarations::{
    analyze_test_declaration, analyze_trait_definition, analyze_trait_impl, analyze_type_definition,
};
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

        // Handle test declarations
        if let Statement::TestDeclaration(test_decl) = stmt {
            analyzed_statements.push(analyze_test_declaration(test_decl, &mut scope)?);
            continue;
        }

        // Use the unified analyze_statement helper for all other statements
        analyzed_statements.extend(analyze_statement(stmt, &mut scope)?);
    }

    Ok(AnalyzedProgram {
        statements: analyzed_statements,
        scope,
    })
}

/// Analyze a single statement (which might produce multiple analyzed statements if it's a block)
pub fn analyze_statement(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Vec<AnalyzedStatement>, GlossaError> {
    // Check for control flow (if, while, etc.)
    if let Some(control_flow) = analyze_control_flow(stmt, scope)? {
        // If it's a function definition, register it in the scope
        if let AnalyzedStatement::FunctionDef {
            name,
            params,
            return_type,
            ..
        } = &control_flow
        {
            let param_types: Vec<GlossaType> = params
                .iter()
                .map(|(_, ty)| ty.clone().unwrap_or(GlossaType::Unknown))
                .collect();
            scope.define_function(name.clone(), param_types, return_type.clone());
        }

        return Ok(vec![control_flow]);
    }

    // Check for struct instantiation pattern
    if let Some(struct_inst) = try_parse_struct_instantiation(stmt, scope)? {
        return Ok(vec![struct_inst]);
    }

    // Check for trait method call pattern
    if let Some(method_call) = try_parse_trait_method_call(stmt, scope)? {
        return Ok(vec![method_call]);
    }

    // Check if it's a block statement (regular statement containing a single block expression)
    if let Some(block_stmts) = extract_block_statements(stmt) {
        let mut analyzed = Vec::new();
        // Create a child scope for the block
        // This ensures variables defined inside the block don't leak out
        let mut block_scope = scope.enter_scope();
        for s in block_stmts {
            analyzed.extend(analyze_statement(s, &mut block_scope)?);
        }
        return Ok(analyzed);
    }

    // Use the assembler-based approach for regular statements
    let assembled = assemble_statement(stmt)?;
    let analyzed = convert_assembled_to_analyzed(&assembled, scope)?;
    Ok(vec![analyzed])
}

fn extract_block_statements(stmt: &Statement) -> Option<&Vec<Statement>> {
    if let Statement::Regular { clauses, .. } = stmt
        && clauses.len() == 1
        && clauses[0].expressions.len() == 1
        && let Expr::Block(stmts) = &clauses[0].expressions[0]
    {
        Some(stmts)
    } else {
        None
    }
}

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
