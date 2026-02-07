//! Semantic Parser for ΓΛΩΣΣΑ
//!
//! This module bridges the gap between the raw text parsing (Grammar) and the
//! high-level program structure (AST).
//!
//! # The Parsing Flow
//!
//! 1. **Grammar (`src/grammar`)**: Uses [`pest`] (PEG parser) to tokenize the input
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

pub mod builder;

use crate::ast::Program;
use crate::errors::GlossaError;
use pest::error::{ErrorVariant, InputLocation};

pub use builder::ParseError;

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
    builder::parse_source(source).map_err(|e| match e {
        ParseError::PestError(pest_err) => {
            let message = match &pest_err.variant {
                ErrorVariant::CustomError { message } => message.clone(),
                ErrorVariant::ParsingError { positives, .. } => {
                    if positives.is_empty() {
                        "Unexpected token".to_string()
                    } else {
                        let expected: Vec<String> = positives
                            .iter()
                            .map(|r| format!("{:?}", r))
                            .collect();
                        format!("Expected one of: {}", expected.join(", "))
                    }
                }
            };

            let span = match pest_err.location {
                InputLocation::Pos(pos) => (pos, 1).into(),
                InputLocation::Span((start, end)) => (start, end - start).into(),
            };

            GlossaError::parse_with_source(message, source, span)
        }
        _ => GlossaError::parse(e.to_string()),
    })
}
