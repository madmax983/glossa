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
use super::patterns::{try_parse_struct_instantiation, try_parse_trait_method_call};
use super::{AnalyzedStatement, GlossaType, Scope, assemble_statement};
use crate::ast::{Expr, Program, Statement};
use crate::errors::GlossaError;

/// Analyzed program with resolved names and types
#[derive(Debug, Clone)]
pub struct AnalyzedProgram {
    pub statements: Vec<AnalyzedStatement>,
    pub scope: Scope,
}

/// The main semantic analyzer struct
pub struct Analyzer;

impl Analyzer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    pub fn analyze(
        &mut self,
        stmt: &Statement,
        scope: &mut Scope,
    ) -> Result<Vec<AnalyzedStatement>, GlossaError> {
        // 1. Check for function definitions
        if contains_function_definition_verb(stmt)
            && let Some(func_def) = parse_function_definition(stmt, scope, self)?
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
        // Pass self as the analyzer
        if let Some(control_flow) = analyze_control_flow(stmt, scope, self)? {
            return Ok(vec![control_flow]);
        }

        // 3. Check for struct instantiation pattern
        if let Some(struct_inst) = try_parse_struct_instantiation(stmt, scope)? {
            return Ok(vec![struct_inst]);
        }

        // 4. Check for trait method call pattern
        if let Some(method_call) = try_parse_trait_method_call(stmt, scope)? {
            return Ok(vec![method_call]);
        }

        // 5. Check if it's a block statement (regular statement containing a single block expression)
        if let Some(block_stmts) = extract_block_statements(stmt) {
            let mut analyzed = Vec::new();
            // Create a child scope for the block
            // This ensures variables defined inside the block don't leak out
            let mut block_scope = scope.enter_scope();
            for s in block_stmts {
                analyzed.extend(self.analyze(s, &mut block_scope)?);
            }
            return Ok(analyzed);
        }

        // 6. Use the assembler-based approach for regular statements
        let assembled = assemble_statement(stmt)?;
        let analyzed = convert_assembled_to_analyzed(&assembled, scope)?;
        Ok(vec![analyzed])
    }
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
/// This is the entry point that instantiates the Analyzer.
pub fn analyze_program(program: &Program) -> Result<AnalyzedProgram, GlossaError> {
    let mut scope = Scope::new();
    let mut analyzed_statements = Vec::new();
    let mut analyzer = Analyzer::new();

    for stmt in &program.statements {
        // Handle type definitions
        if let Statement::TypeDefinition(type_def) = stmt {
            analyzed_statements.push(analyze_type_definition(type_def, &mut scope)?);
            continue;
        }

        // Handle trait definitions
        if let Statement::TraitDefinition(trait_def) = stmt {
            analyzed_statements.push(analyze_trait_definition(
                trait_def,
                &mut scope,
                &mut analyzer,
            )?);
            continue;
        }

        // Handle trait implementations
        if let Statement::TraitImpl(trait_impl) = stmt {
            analyzed_statements.push(analyze_trait_impl(trait_impl, &mut scope, &mut analyzer)?);
            continue;
        }

        // Handle test declarations
        if let Statement::TestDeclaration(test_decl) = stmt {
            analyzed_statements.push(analyze_test_declaration(
                test_decl,
                &mut scope,
                &mut analyzer,
            )?);
            continue;
        }

        // Use the analyzer for all other statements
        analyzed_statements.extend(analyzer.analyze(stmt, &mut scope)?);
    }

    let analyzed = AnalyzedProgram {
        statements: analyzed_statements,
        scope,
    };

    crate::semantic::validation::validate_program(&analyzed)?;

    Ok(analyzed)
}
