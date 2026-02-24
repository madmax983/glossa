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

/// Check recursion depth to prevent stack overflows
///
/// This function performs a fast linear scan of the source code to ensure that
/// parentheses, braces, and brackets are not nested deeper than `MAX_DEPTH` (500).
/// This prevents stack overflows during the recursive parsing phase.
pub(crate) fn check_recursion_depth(source: &str) -> Result<(), ParseError> {
    const MAX_DEPTH: usize = 500;
    let mut depth = 0;
    let mut in_string = false;
    let bytes = source.as_bytes();
    let mut i = 0;

    // Optimization: Iterate bytes directly to avoid expensive UTF-8 decoding of Greek characters.
    // We only care about structural characters which are ASCII (except for « and »).
    // « is [0xC2, 0xAB]
    // » is [0xC2, 0xBB]
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            // Check for » [0xC2, 0xBB]
            if b == 0xC2 && i + 1 < bytes.len() && bytes[i + 1] == 0xBB {
                in_string = false;
                i += 2;
            } else {
                i += 1;
            }
        } else {
            match b {
                // Check for « [0xC2, 0xAB]
                0xC2 => {
                    if i + 1 < bytes.len() && bytes[i + 1] == 0xAB {
                        in_string = true;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                b'(' | b'{' | b'[' => {
                    depth += 1;
                    if depth > MAX_DEPTH {
                        return Err(ParseError::RecursionLimitExceeded(MAX_DEPTH));
                    }
                    i += 1;
                }
                b')' | b'}' | b']' => {
                    depth = depth.saturating_sub(1);
                    i += 1;
                }
                b'/' => {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                        // Skip comment
                        i += 2;
                        while i < bytes.len() {
                            let c = bytes[i];
                            i += 1;
                            if c == b'\n' || c == b'\r' {
                                break;
                            }
                        }
                    } else {
                        i += 1;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recursion_limit_exceeded() {
        // 501 nested parentheses
        let source = "(".repeat(501) + &")".repeat(501);
        let result = check_recursion_depth(&source);
        assert!(matches!(
            result,
            Err(ParseError::RecursionLimitExceeded(500))
        ));
    }

    #[test]
    fn test_recursion_limit_not_exceeded() {
        // 500 nested parentheses (should pass check, though pest might fail to parse empty parens)
        let source = "(".repeat(500) + &")".repeat(500);
        // We only care about the recursion check here
        let result = check_recursion_depth(&source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recursion_limit_ignored_in_string() {
        // Parentheses inside string literal shouldn't count
        let source = "«".to_string() + &"(".repeat(600) + "»";
        let result = check_recursion_depth(&source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recursion_limit_ignored_in_comment() {
        // Parentheses inside comment shouldn't count
        let source = "// ".to_string() + &"(".repeat(600);
        let result = check_recursion_depth(&source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recursion_limit_mixed_brackets() {
        // Mixed brackets should all count towards the same limit
        // 200 (, 200 {, 101 [ = 501 total
        let source = "(".repeat(200)
            + &"{".repeat(200)
            + &"[".repeat(101)
            + &"]".repeat(101)
            + &"}".repeat(200)
            + &")".repeat(200);
        let result = check_recursion_depth(&source);
        assert!(matches!(
            result,
            Err(ParseError::RecursionLimitExceeded(500))
        ));
    }

    #[test]
    fn test_recursion_limit_unbalanced_but_safe() {
        // Unbalanced brackets that don't exceed depth
        // (((...))) then (((...))) - sequential, not nested
        let part = "(".repeat(400) + &")".repeat(400);
        let source = part.clone() + &part;
        let result = check_recursion_depth(&source);
        assert!(result.is_ok());
    }
}
