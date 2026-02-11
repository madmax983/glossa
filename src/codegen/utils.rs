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
/// into valid ASCII Rust identifiers. It uses a combination of name mapping
/// (for single letters like `α`) and transliteration (for words like `χρήστης`).
///
/// # Edge Cases
///
/// Characters that do not have a standard Latin mapping (like `ϟ` Koppa) are
/// hex-encoded to ensure uniqueness and prevent collisions.
///
/// * `ϟ` -> `_u3df_`
///
/// # Examples
///
/// ```
/// // These are internal functions, but here is how they behave:
/// // sanitize_name("ξ") -> "xi"
/// // sanitize_name("χρήστης") -> "chrestes"
/// ```
pub(crate) fn sanitize_name(name: &str) -> String {
    // Directly transliterate without special casing single letters
    // This prevents collisions between single letters and their full names
    // e.g. "σ" (sigma) vs "σίγμα" (sigma)
    // Prefix with "g_" to namespace all user-defined identifiers and avoid collisions with Rust keywords
    format!("g_{}", transliterate(name))
}

/// Transliterate Greek to Latin characters
pub(crate) fn transliterate(greek: &str) -> String {
    let mut result = String::new();

    for c in greek.chars() {
        let trans = match c {
            // Greek characters are prefixed with "_" to avoid collision with ASCII identifiers
            // e.g. "α" -> "_a", but "a" -> "a"
            'α' => "_a",
            'β' => "_b",
            'γ' => "_g",
            'δ' => "_d",
            'ε' => "_e",
            'ζ' => "_z",
            'η' => "_h", // Distinct from 'e' (epsilon)
            'θ' => "_th",
            'ι' => "_i",
            'κ' => "_k",
            'λ' => "_l",
            'μ' => "_m",
            'ν' => "_n",
            'ξ' => "_x",
            'ο' => "_o",
            'π' => "_p",
            'ρ' => "_r",
            'σ' | 'ς' => "_s",
            'τ' => "_t",
            'υ' => "_u",
            'φ' => "_ph",
            'χ' => "_ch",
            'ψ' => "_ps",
            'ω' => "_w", // Distinct from 'o' (omicron)
            // Underscore is escaped to "__" to avoid collision with prefixed Greek chars
            // e.g. "_a" -> "__a", but "α" -> "_a"
            '_' => "__",
            _ => {
                // Keep only ASCII alphanumeric characters
                if c.is_ascii_alphanumeric() {
                    result.push(c);
                } else {
                    // Replace invalid characters with unique hex code to prevent collisions
                    // e.g. ϟ -> _u3df_
                    // This is safe from collision because:
                    // 1. Literal underscores '_' map to '__'
                    // 2. Greek characters map to '_x' (single underscore)
                    // 3. Hex encoding uses '_u..._' (single underscore + 'u')
                    // 4. Literal 'u' in input maps to 'u'
                    //
                    // Example:
                    // Koppa (ϟ) -> _u3df_
                    // User input "_u3df_" -> __u3df__
                    // User input "u3df" -> u3df
                    // Greek Upsilon + "3df" -> _u3df (no trailing underscore)
                    use std::fmt::Write;
                    write!(result, "_u{:x}_", c as u32).unwrap();
                }
                continue;
            }
        };
        result.push_str(trans);
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
        assert_eq!(sanitize_name("ξ"), "g__x");
        assert_eq!(sanitize_name("α"), "g__a");
        assert_eq!(sanitize_name("ω"), "g__w");
    }

    #[test]
    fn test_transliterate() {
        // χ (chi) -> _ch, ρ -> _r, η -> _h, σ -> _s, τ -> _t, ο -> _o, ς -> _s
        assert_eq!(transliterate("χρηστος"), "_ch_r_h_s_t_o_s");
        assert_eq!(transliterate("λογος"), "_l_o_g_o_s");
        // φ (phi) -> _ph
        assert_eq!(transliterate("φιλοσοφια"), "_ph_i_l_o_s_o_ph_i_a");
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
        let input = "αϟβ";
        let output = transliterate(input);
        assert_eq!(output, "_a_u3df__b");
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

    #[test]
    fn test_transliterate_collision() {
        // These currently fail because transliterate("α") == "a" == transliterate("a")
        assert_ne!(
            transliterate("α"),
            transliterate("a"),
            "Alpha should not collide with a"
        );
        assert_ne!(
            transliterate("η"),
            transliterate("h"),
            "Eta should not collide with h"
        );
        assert_ne!(
            transliterate("ω"),
            transliterate("w"),
            "Omega should not collide with w"
        );
        assert_ne!(
            transliterate("σ"),
            transliterate("s"),
            "Sigma should not collide with s"
        );
        assert_ne!(
            transliterate("αa"),
            transliterate("aa"),
            "Mixed alpha should not collide"
        );
    }

    #[test]
    fn test_transliterate_underscore() {
        // _ maps to __
        assert_eq!(transliterate("_"), "__");
        assert_eq!(transliterate("a_b"), "a__b");
    }

    #[test]
    fn test_transliterate_ascii_u() {
        // u maps to u (ensure no collision with hex prefix _u)
        assert_eq!(transliterate("u"), "u");
        // Check collision with hypothetical hex code
        // Koppa (ϟ) -> _u3df_
        // "u3df" -> u3df
        assert_ne!(transliterate("ϟ"), transliterate("u3df"));
        // "_u3df_" -> __u3df__
        assert_ne!(transliterate("ϟ"), transliterate("_u3df_"));
    }

    #[test]
    fn test_transliterate_invalid_unicode() {
        // ⚡ (High Voltage) -> _u26a1_
        // 26A1 is the hex code
        assert_eq!(transliterate("⚡"), "_u26a1_");
    }

    #[test]
    fn test_sanitize_empty() {
        assert_eq!(sanitize_name(""), "g__var_empty");
    }

    #[test]
    fn test_sanitize_numeric_start() {
        // "123" -> "_123" (via transliterate logic at end of function)
        // sanitize_name prefixes with g_, so g_ + _123 = g__123
        // Wait, sanitize_name calls transliterate.
        // transliterate("123"):
        //   result = "123".
        //   result starts with numeric -> returns "_123".
        // sanitize_name adds "g_".
        // So "g__123".
        assert_eq!(sanitize_name("123"), "g__123");
    }
}
