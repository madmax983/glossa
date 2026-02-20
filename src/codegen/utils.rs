//! Utility functions for code generation

/// Capitalize the first letter of a string (for Rust type/trait names)
pub(crate) fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Sanitize a Greek name for use as a Rust identifier
///
/// This function performs the critical step of converting Ancient Greek identifiers
/// into valid ASCII Rust identifiers. It exclusively uses hex-encoding for all
/// non-ASCII characters to guarantee uniqueness and prevent collisions.
///
/// # The Strategy: Hex Encoding
///
/// Instead of trying to map Greek letters to Latin letters (which is lossy and
/// prone to collisions like `χ` -> `ch` vs `c` + `h`), we simply hex-encode
/// the Unicode scalar value of every non-ASCII character.
///
/// * `α` (U+03B1) -> `_u3b1_`
/// * `ξ` (U+03BE) -> `_u3be_`
///
/// This ensures that `x` (ASCII) and `ξ` (Greek Xi) are distinct in the generated Rust code.
///
/// # Examples
///
/// ```rust
/// use glossa::codegen::sanitize_name;
///
/// // Hex encoding (prefixed with g_ for namespace safety)
/// assert_eq!(sanitize_name("ξ"), "g__u3be_");
/// assert_eq!(sanitize_name("χρηστης"), "g__u3c7__u3c1__u3b7__u3c3__u3c4__u3b7__u3c2_");
///
/// // Keyword safety (even Rust keywords are safe due to g_ prefix)
/// assert_eq!(sanitize_name("if"), "g_if");
/// ```
pub fn sanitize_name(name: &str) -> String {
    // Directly transliterate without special casing single letters
    // This prevents collisions between single letters and their full names
    // e.g. "σ" (sigma) vs "σίγμα" (sigma)
    // Prefix with "g_" to namespace all user-defined identifiers and avoid collisions with Rust keywords
    format!("g_{}", transliterate(name))
}

/// Transliterate Greek to Latin characters via Hex Encoding
///
/// This function maps Greek characters (and any non-ASCII character) to a
/// hex-encoded sequence `_uXXXX_`. It ensures that the output contains only
/// valid Rust identifier characters (alphanumeric + underscore).
///
/// **Note:** This function expects normalized (monotonic) Greek text, but will
/// work correctly (by hex-encoding) on any input.
///
/// # Mapping Strategy
///
/// * **ASCII Alphanumeric + `_`**: Kept as-is.
/// * **Everything else**: Hex-encoded as `_uXXXX_`.
///
/// This strategy is "lossless" for identifiers and guarantees no collisions.
///
/// # Examples
///
/// ```rust
/// use glossa::codegen::transliterate;
///
/// assert_eq!(transliterate("λογος"), "_u3bb__u3bf__u3b3__u3bf__u3c2_");
/// assert_eq!(transliterate("φιλοσοφια"), "_u3c6__u3b9__u3bb__u3bf__u3c3__u3bf__u3c6__u3b9__u3b1_");
/// ```
pub fn transliterate(greek: &str) -> String {
    let mut result = String::new();

    for c in greek.chars() {
        // We now hex-encode ALL Greek characters to avoid collisions with ASCII.
        // Previously, 'ξ' mapped to 'x', causing collision with ASCII 'x'.
        // Now, 'ξ' maps to '_u3be_', which is distinct from 'x' (which stays 'x').
        if c.is_ascii_alphanumeric() || c == '_' {
            result.push(c);
        } else {
            // Replace invalid characters with unique hex code to prevent collisions
            use std::fmt::Write;
            write!(result, "_u{:x}_", c as u32).unwrap();
        }
    }

    // Ensure it starts with a letter or underscore (valid Rust identifier)
    if result.is_empty() {
        return "_var_empty".to_string();
    }

    if result
        .chars()
        .next()
        .map(|c| c.is_numeric())
        .unwrap_or(false)
    {
        format!("_{}", result)
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_greek_letter() {
        assert_eq!(sanitize_name("ξ"), "g__u3be_");
        assert_eq!(sanitize_name("α"), "g__u3b1_");
        assert_eq!(sanitize_name("ω"), "g__u3c9_");
    }

    #[test]
    fn test_transliterate() {
        // All Greek characters are now hex encoded to avoid ASCII collisions
        // χ (chi) -> _u3c7_
        // ρ -> _u3c1_
        // η -> _u3b7_
        // ...
        assert_eq!(
            transliterate("χρηστος"),
            "_u3c7__u3c1__u3b7__u3c3__u3c4__u3bf__u3c2_"
        );
        assert_eq!(transliterate("λογος"), "_u3bb__u3bf__u3b3__u3bf__u3c2_");
        assert_eq!(
            transliterate("φιλοσοφια"),
            "_u3c6__u3b9__u3bb__u3bf__u3c3__u3bf__u3c6__u3b9__u3b1_"
        );
    }

    #[test]
    fn test_transliterate_unique() {
        // Test that different invalid characters produce different outputs
        let koppa = "ϟ";
        let stigma = "ϛ";

        let t_koppa = transliterate(koppa);
        let t_stigma = transliterate(stigma);

        assert_ne!(
            t_koppa, t_stigma,
            "Different invalid chars should not collide"
        );
        assert!(t_koppa.contains("_u3df_")); // Koppa is 0x3DF
        assert!(t_stigma.contains("_u3db_")); // Stigma is 0x3DB
    }

    #[test]
    fn test_transliterate_mixed_valid_invalid() {
        // Test mixing valid and invalid characters
        // α -> _u3b1_
        // ϟ -> _u3df_
        // β -> _u3b2_
        let input = "αϟβ";
        let output = transliterate(input);
        assert_eq!(output, "_u3b1__u3df__u3b2_");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("Hello"), "Hello");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("x"), "X");
    }

    #[test]
    fn test_sanitize_keywords_and_prefix() {
        // Test that keywords are safe (by prefixing)
        // If "if" stays "if", it's invalid Rust
        assert_eq!(sanitize_name("if"), "g_if");
        assert_eq!(sanitize_name("fn"), "g_fn");

        // Test that regular identifiers are prefixed
        // This ensures a unique namespace for user variables
        assert_eq!(sanitize_name("x"), "g_x");
        assert_eq!(sanitize_name("foo"), "g_foo");
    }
}
