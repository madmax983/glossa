//! Conversion from assembled statements to analyzed statements
//!
//! This module acts as the "interpreter" of the assembled semantic structure.
//! While the `Assembler` ensures grammatical correctness (Subject-Verb agreement),
//! this module assigns *meaning* to the grammatical structures.

pub(crate) mod classify;
pub(crate) mod extract;
#[cfg(test)]
mod tests;

pub use classify::classify_assembled_statement;

#[cfg(test)]
pub use extract::extract_value;

use crate::errors::GlossaError;
use crate::semantic::AnalyzedStatement;
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::resolver::Scope;

pub fn convert_assembled_to_analyzed(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    classify_assembled_statement(asm_stmt, scope)
}
