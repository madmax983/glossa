//! Conversion from assembled statements to analyzed statements
//!
//! This module acts as the "interpreter" of the assembled semantic structure.
//! While the [`Assembler`](crate::semantic::Assembler) ensures grammatical correctness (Subject-Verb agreement),
//! this module assigns *meaning* to the grammatical structures.
//!
//! # The Interpreter Pattern
//!
//! The conversion process is essentially an interpretation step. It takes a
//! grammatically valid but semantically ambiguous "Assembled Statement" and
//! converts it into a typed, unambiguous "Analyzed Statement" (part of the HIR).
//!
//! This is where "word order independence" meets "semantic meaning".
//!
//! # Pattern Detection Strategy
//!
//! The [`classify_assembled_statement`] function uses a combination of strategies to
//! understand the statement's intent, checking patterns in a specific heuristic order:
//!
//! 1. **Pattern Delegation**: Complex patterns are delegated first.
//!    - **Iterator Chains**: `detect_iterator_pattern` (e.g., `list doubling print`).
//!    - **Property Access**: `classify_property_access_print` (e.g., `user.name print`).
//!    - **Struct Instantiation**: `try_parse_struct_instantiation` (e.g., `x new User ... let`).
//!    - **Function Calls**: `classify_function_call` (e.g., `my_func arg1 arg2 call`).
//!
//! 2. **Verb-Based Classification**: If no complex pattern matches, the main verb drives the logic.
//!    - **Binding** (`ἔστω`): `let x = value`.
//!    - **Assignment** (`γίγνεται`): `x = value`.
//!    - **Collection Ops** (`ὠθεῖ`, `ἕλκεται`, `τίθησι`): `push`, `pop`, `insert`.
//!    - **Print** (`λέγε`, `γράφε`): `println!`.
//!    - **Query** (`?`): Expressions ending in `?`.
//!
//! 3. **Expression Fallback**: If no verb implies a statement, it's treated as a pure expression.
//!    - **Operations**: `1 + 2`.
//!    - **Try/Propagate**: `expr;` (becomes `expr?`).

pub mod assertions;
pub mod bindings;
pub mod collections;
pub mod expressions;
pub mod functions;
pub mod iterators;
pub mod print;

use crate::errors::GlossaError;
use crate::semantic::{AnalyzedStatement, AssembledStatement, Scope};

use self::assertions::{
    classify_assertion, classify_equality_assertion, classify_subjunctive_comparison,
};
use self::bindings::{classify_assignment, classify_variable_binding};
use self::collections::classify_collection_mutation;
use self::expressions::classify_expression;
use self::functions::{classify_function_call, classify_genitive_method_call};
use self::iterators::classify_iterator_pattern;
use self::print::{classify_print, classify_property_access_print, classify_query};

/// Convert an AssembledStatement to an AnalyzedStatement
///
/// This is the main entry point for lowering the "Assembled" semantic model (slot-based)
/// to the "Analyzed" model (HIR/AST-like).
///
/// # Arguments
///
/// * `asm_stmt` - The assembled statement from the `Assembler`.
/// * `scope` - The current semantic scope (for variable lookup and definition).
pub fn convert_assembled_to_analyzed(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    classify_assembled_statement(asm_stmt, scope)
}

/// Classify an assembled statement and extract analyzed expressions
///
/// This function implements the heuristic priority list described in the module-level documentation.
pub fn classify_assembled_statement(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    if let Some(res) = classify_iterator_pattern(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_property_access_print(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_function_call(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_genitive_method_call(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_assertion(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_equality_assertion(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_subjunctive_comparison(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_variable_binding(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_assignment(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_collection_mutation(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_print(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_query(asm_stmt, scope)? {
        return Ok(res);
    }

    classify_expression(asm_stmt)
}
