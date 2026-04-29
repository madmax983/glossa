//! Core Semantic Analyzer
//!
//! This module contains the `Analyzer` struct which orchestrates the semantic analysis
//! process, handling everything from function definitions to control flow and pattern matching.

use super::control_flow::analyze_control_flow;
use super::conversion::convert_assembled_to_analyzed;
use super::declarations::{
    analyze_test_declaration, analyze_trait_definition, analyze_trait_impl,
    analyze_type_definition, parse_function_definition,
};
use super::expressions::contains_function_definition_verb;
use super::patterns::{try_parse_method_call, try_parse_struct_instantiation};
use super::{AnalyzedStatement, GlossaType, Scope, assemble_statement};
use crate::ast::{Expr, Program, Statement};
use crate::errors::GlossaError;

/// Analyzed program with resolved names and types
///
/// This structure represents the ultimate truth (τέλος) of the user's intent,
/// having successfully passed through syntax parsing and morphological analysis.
/// It is completely devoid of ambiguity and is fully prepared for translation
/// into Rust code (codegen).
///
/// ## Examples
///
/// ```rust
/// use glossa::ast::Program;
/// use glossa::semantic::analyze_program;
///
/// // Create an empty program structure
/// let empty_ast = Program { statements: vec![] };
///
/// // The analyzed program will contain an empty scope and no statements
/// let analyzed = analyze_program(&empty_ast).unwrap();
/// assert!(analyzed.statements.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct AnalyzedProgram {
    /// The linear sequence of semantically valid statements ready for codegen
    pub statements: Vec<AnalyzedStatement>,
    /// The global execution scope containing known bindings, variables, and type definitions
    pub scope: Scope,
}

/// Semantically analyze a single statement and update the scope environment.
///
/// This function acts as the entry point for understanding *what* the AST is trying
/// to express. It systematically checks against known high-level constructs
/// (Function definitions, Struct instantiations, Control flow).
/// If it matches none of those patterns, it falls back to the `Assembler` which
/// attempts to assemble a meaningful sentence from individual words.
///
/// # Returns
/// A `Vec<AnalyzedStatement>` because a single block statement can return
/// multiple analyzed statements internally when expanded in scope.
///
/// # Examples
///
/// ```rust
/// use glossa::parser::parse;
/// use glossa::semantic::{Scope, analyze_statement};
///
/// let mut scope = Scope::new();
/// let ast = parse("ξ πέντε ἔστω.").unwrap(); // "Let ξ be 5."
///
/// let statements = analyze_statement(&ast.statements[0], &mut scope).unwrap();
///
/// assert_eq!(statements.len(), 1);
/// assert!(scope.lookup_binding("ξ").is_some());
/// ```
pub fn analyze_statement(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Vec<AnalyzedStatement>, GlossaError> {
    analyze_statement_recursive(stmt, scope, 0)
}

fn analyze_statement_recursive(
    stmt: &Statement,
    scope: &mut Scope,
    depth: usize,
) -> Result<Vec<AnalyzedStatement>, GlossaError> {
    if depth > crate::limits::MAX_AST_DEPTH {
        return Err(GlossaError::semantic(
            "Recursion limit exceeded in statement analysis",
        ));
    }

    // 1. Check for function definitions
    if contains_function_definition_verb(stmt)
        && let Some(func_def) = parse_function_definition(stmt, scope)?
    {
        // Register the function in the scope
        if let AnalyzedStatement::FunctionDef {
            name,
            params,
            return_type,
            ..
        } = &func_def
        {
            let param_types: Vec<GlossaType> = params
                .iter()
                .map(|(_, ty)| ty.clone().unwrap_or(GlossaType::Unknown))
                .collect();
            scope.define_function(name.clone(), param_types, return_type.clone());
        }
        return Ok(vec![func_def]);
    }

    // 2. Check for control flow (if, while, etc.)
    if let Some(control_flow) = analyze_control_flow(stmt, scope)? {
        return Ok(vec![control_flow]);
    }

    // 3. Check for struct instantiation pattern
    if let Some(struct_inst) = try_parse_struct_instantiation(stmt, scope)? {
        return Ok(vec![struct_inst]);
    }

    // 4. Check for standalone method call pattern
    if let Some(method_call) = try_parse_method_call(stmt, scope)? {
        return Ok(vec![method_call]);
    }

    // 5. Check if it's a block statement (regular statement containing a single block expression)
    if let Some(block_stmts) = extract_block_statements(stmt) {
        // ⚡ Bolt Optimization: Uses `Vec::with_capacity` based on the block statements length to prevent reallocation.
        let mut analyzed = Vec::with_capacity(block_stmts.len());
        // Create a child scope for the block
        // This ensures variables defined inside the block don't leak out
        return scope.with_scope(|block_scope| {
            for s in block_stmts {
                analyzed.extend(analyze_statement_recursive(s, block_scope, depth + 1)?);
            }
            Ok(analyzed)
        });
    }

    // 6. Use the assembler-based approach for regular statements
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

/// Perform semantic analysis on a program
///
/// This is the entry point for semantic analysis.
pub fn analyze_program(program: &Program) -> Result<AnalyzedProgram, GlossaError> {
    crate::semantic::validation::check_program_depth(program)?;

    let mut scope = Scope::new();
    // ⚡ Bolt Optimization: Uses `Vec::with_capacity` based on the program statements length to prevent reallocation.
    let mut analyzed_statements = Vec::with_capacity(program.statements.len());

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

        // Use the analyzer for all other statements
        analyzed_statements.extend(analyze_statement(stmt, &mut scope)?);
    }

    let analyzed = AnalyzedProgram {
        statements: analyzed_statements,
        scope,
    };

    crate::semantic::validation::validate_program(&analyzed)?;

    Ok(analyzed)
}
