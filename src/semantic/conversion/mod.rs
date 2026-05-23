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

use super::expressions::{
    analyze_argument_expr, build_binary_expr, build_expressions_from_literals_and_ops,
    literal_to_analyzed_expr,
};
use crate::ast::Expr;
use crate::errors::GlossaError;
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::model::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement};
use crate::semantic::resolver::Scope;

use crate::semantic::types::GlossaType;

pub(crate) mod classification;
pub(crate) mod extraction;
#[cfg(test)]
mod tests;
pub use classification::classify_assembled_statement;

/// Convert an AssembledStatement to an AnalyzedStatement
///
/// This is the main entry point for lowering the "Assembled" semantic model (slot-based)
/// to the "Analyzed" model (HIR/AST-like).
///
/// Evaluates and translates a grammatically sound statement into the semantically typed AST.
///
/// This serves as the top-level interpreter connecting the raw output of the
/// [`crate::semantic::Assembler`] (`AssembledStatement`) with the High-Level Intermediate
/// Representation (`AnalyzedStatement`). It assigns concrete meaning to grammatical roles
/// (e.g., assigning a Subject the role of "Variable Name").
///
/// # Arguments
///
/// * `asm_stmt` - The assembled statement from the `Assembler`.
/// * `scope` - The current semantic scope (for variable lookup and definition).
///
/// # Examples
///
/// ```rust,ignore
/// // Example cannot be run as a doctest because this module is pub(crate)
/// use glossa::semantic::assembly::AssembledStatement;
/// use glossa::semantic::conversion::convert_assembled_to_analyzed;
/// use glossa::semantic::resolver::Scope;
/// use glossa::ast::{Expr, Word};
/// use glossa::morphology::lexicon::{LexiconEntry, VerbType};
///
/// let mut scope = Scope::new();
/// let mut asm = AssembledStatement::new();
///
/// // Simulate: "«χαῖρε» λέγε."
/// asm.verb = Some(LexiconEntry {
///     lemma: "λεγω".into(),
///     english_equivalent: "say".into(),
///     part_of_speech: glossa::morphology::lexicon::PartOfSpeech::Verb(VerbType::Transitive),
/// });
/// asm.strings.push("χαῖρε".into());
///
/// let result = convert_assembled_to_analyzed(&asm, &mut scope);
/// assert!(result.is_ok());
/// ```
pub fn convert_assembled_to_analyzed(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    classify_assembled_statement(asm_stmt, scope)
}

#[cfg(test)]
pub use extraction::extract_value;
