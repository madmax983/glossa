//! Semantic Parser for ΓΛΩΣΣΑ
//!
//! This module handles the conversion from the PEG concrete syntax tree (CST)
//! to the Abstract Syntax Tree (AST).

pub mod builder;

use crate::ast::Program;
use crate::errors::GlossaError;

pub use builder::ParseError;

/// Parse a ΓΛΩΣΣΑ source string into an AST
pub fn parse(source: &str) -> Result<Program, GlossaError> {
    builder::parse_source(source).map_err(|e| GlossaError::parse(e.to_string()))
}
