//! Semantic Parser for ΓΛΩΣΣΑ
//!
//! This module bridges the gap between the raw text parsing (Grammar) and the
//! high-level program structure (AST).
//!
//! # The Parsing Flow
//!
//! 1. **Grammar (`glossa.pest`)**: Uses [`pest`] (PEG parser) to tokenize the input
//!    and verify it matches the language rules. This produces a "Concrete Syntax Tree" (CST)
//!    of untyped pairs (e.g., `Rule::greek_word`, `Rule::number_literal`).
//!
//! 2. **Builder (`src/parser/builder.rs`)**: Walks this CST and constructs strongly-typed
//!    [`crate::ast`] nodes. This is where we handle:
//!    * Text normalization (converting `Ἀθῆναι` to `αθηναι`)
//!    * Number parsing
//!    * Structural validation (e.g., ensuring a trait method has a name)
//!
//! # Why separate Grammar and Builder?
//!
//! Separating the PEG grammar from the AST construction allows us to:
//! * Keep the grammar file (`glossa.pest`) clean and readable.
//! * Handle complex logic (like normalization) in Rust code, not in the grammar.
//! * Provide better error messages during the conversion phase.

pub(crate) mod builder;
pub mod numerals;

use crate::ast::Program;
use crate::errors::GlossaError;
use pest_derive::Parser;

pub use builder::ParseError;

#[derive(Parser)]
#[grammar = "parser/glossa.pest"]
pub struct GlossaParser;

impl From<ParseError> for GlossaError {
    fn from(err: ParseError) -> Self {
        GlossaError::parse(err.to_string())
    }
}

/// Parse a ΓΛΩΣΣΑ source string into an AST
///
/// This is the main entry point for the parsing phase.
///
/// # Examples
///
/// ```
/// use glossa::parser::parse;
///
/// let source = "«χαῖρε» λέγε.";
/// let program = parse(source).unwrap();
/// assert_eq!(program.statements.len(), 1);
/// ```
pub fn parse(source: &str) -> Result<Program, GlossaError> {
    builder::parse_source(source).map_err(GlossaError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn test_parse_hello_cosmos() {
        let source = "«χαῖρε κόσμε» λέγε.";
        let result = GlossaParser::parse(Rule::program, source);
        assert!(
            result.is_ok(),
            "Failed to parse hello cosmos: {:?}",
            result.err()
        );

        let pairs = result.unwrap();
        // Verify we got a program with at least one statement
        let program = pairs.into_iter().next().unwrap();
        assert_eq!(program.as_rule(), Rule::program);
    }
}
