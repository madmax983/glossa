//! Common parser definitions

use crate::parser::numerals;

/// Parse a number literal or a greek numeral
pub fn parse_number_literal(text: &str) -> Result<i64, ParseError> {
    if let Ok(val) = text.parse::<i64>() {
        Ok(val)
    } else {
        numerals::parse_greek_numeral(text)
            .map_err(|e| ParseError::InvalidNumber(format!("{} - {}", text, e)))
    }
}

/// Errors that can occur during AST construction
///
/// These errors signify that the user's manuscript (source code) cannot be deciphered
/// by the parser, either because it contains invalid grammatical structures or because
/// it exceeds the mortal limits of computation (e.g. recursion depth).
///
/// ## Examples
///
/// ```rust
/// use glossa::parser::ParseError;
///
/// // Create an error when encountering an impossibly large number
/// let err = ParseError::InvalidNumber("12345678901234567890".to_string());
/// assert_eq!(err.to_string(), "Invalid number: 12345678901234567890");
/// ```
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    /// A raw error emitted directly from the `pest` grammar parser (Concrete Syntax Tree)
    #[error("Parse error: {0}")]
    PestError(String),

    /// Missing or unidentifiable term when an expression was required
    #[error("Empty term in expression")]
    EmptyTerm,

    /// Failed to parse a string literal into a valid number or Greek numeral
    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    /// Encountered a PEG rule that the AST builder doesn't know how to handle
    #[error("Unexpected rule: {0}")]
    UnexpectedRule(String),

    /// Prevented a stack overflow by aborting deeply nested structure parsing
    #[error("Recursion limit exceeded: depth > {0}")]
    RecursionLimitExceeded(usize),
}
