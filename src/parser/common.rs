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
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    PestError(String),

    #[error("Empty term in expression")]
    EmptyTerm,

    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    #[error("Unexpected rule: {0}")]
    UnexpectedRule(String),

    #[error("Recursion limit exceeded: depth > {0}")]
    RecursionLimitExceeded(usize),
}
