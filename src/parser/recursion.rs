use super::ParseError;

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
