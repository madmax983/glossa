use super::ParseError;

/// Check recursion depth to prevent stack overflows
///
/// This function performs a fast linear scan of the source code to ensure that
/// parentheses, braces, brackets, AND keywords that induce nesting (like `δοκιμή`)
/// are not nested deeper than `MAX_DEPTH` (500).
/// This prevents stack overflows during the recursive parsing phase.
pub(crate) fn check_recursion_depth(source: &str) -> Result<(), ParseError> {
    use crate::limits::MAX_PARSE_DEPTH;
    let mut depth = 0;
    let mut in_string = false;
    let bytes = source.as_bytes();
    let mut i = 0;

    // Optimization: Iterate bytes directly to avoid expensive UTF-8 decoding of Greek characters.
    // We only care about structural characters which are ASCII (except for « and »)
    // and specific keywords (`δοκιμή`, `τέλος`) that affect nesting.
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
            // Check for keywords first if byte matches start of UTF-8 sequence
            // δοκιμή (test): starts with \xCE\xB4 (δ)
            // τέλος (end): starts with \xCF\x84 (τ)

            // Check for "δοκιμή" or "δοκιμη" (test declaration start)
            if b == 0xCE && (source[i..].starts_with("δοκιμή") || source[i..].starts_with("δοκιμη"))
            {
                depth += 1;
                if depth > MAX_PARSE_DEPTH {
                    return Err(ParseError::RecursionLimitExceeded(MAX_PARSE_DEPTH));
                }
                // Both versions have length 12 bytes
                i += "δοκιμή".len();
                continue;
            }

            // Check for "τέλος" or "τελος" (test declaration end)
            if b == 0xCF && (source[i..].starts_with("τέλος") || source[i..].starts_with("τελος"))
            {
                depth = depth.saturating_sub(1);
                // Both versions have length 10 bytes
                i += "τέλος".len();
                continue;
            }

            match b {
                // Check for « [0xC2, 0xAB]
                0xC2 if i + 1 < bytes.len() && bytes[i + 1] == 0xAB => {
                    in_string = true;
                    i += 2;
                }
                b'(' | b'{' | b'[' => {
                    depth += 1;
                    if depth > MAX_PARSE_DEPTH {
                        return Err(ParseError::RecursionLimitExceeded(MAX_PARSE_DEPTH));
                    }
                    i += 1;
                }
                b')' | b'}' | b']' => {
                    depth = depth.saturating_sub(1);
                    i += 1;
                }
                b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                    // Skip comment
                    i += 2;
                    while i < bytes.len() {
                        let c = bytes[i];
                        i += 1;
                        if c == b'\n' || c == b'\r' {
                            break;
                        }
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
