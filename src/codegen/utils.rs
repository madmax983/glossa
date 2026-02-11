//! Utility functions for code generation

use crate::semantic::GlossaType;

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
            'α' => "a",
            'β' => "b",
            'γ' => "g",
            'δ' => "d",
            'ε' => "e",
            'ζ' => "z",
            'η' => "h", // Distinct from 'e' (epsilon)
            'ι' => "i",
            'κ' => "k",
            'λ' => "l",
            'μ' => "m",
            'ν' => "n",
            'ξ' => "x",
            'ο' => "o",
            'π' => "p",
            'ρ' => "r",
            'σ' | 'ς' => "s",
            'τ' => "t",
            'υ' => "u",
            'ω' => "w", // Distinct from 'o' (omicron)
            // Digraphs and other characters are hex-encoded to prevent collisions
            // θ, φ, χ, ψ map to _u..._ because th, ph, ch, ps collide with sequences
            _ => {
                // Keep only ASCII alphanumeric characters and underscore
                if c.is_ascii_alphanumeric() || c == '_' {
                    result.push(c);
                } else {
                    // Replace invalid characters with unique hex code to prevent collisions
                    // e.g. ϟ -> _u3df_
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

/// Convert a Glossa type to its Rust equivalent string
pub fn to_rust_type(ty: &GlossaType) -> String {
    match ty {
        GlossaType::Number => "i64".to_string(),
        GlossaType::String => "String".to_string(),
        GlossaType::Boolean => "bool".to_string(),
        GlossaType::List(inner) => format!("Vec<{}>", to_rust_type(inner)),
        GlossaType::Set(inner) => format!("HashSet<{}>", to_rust_type(inner)),
        GlossaType::Map(key, value) => {
            format!("HashMap<{}, {}>", to_rust_type(key), to_rust_type(value))
        }
        GlossaType::Option(inner) => format!("Option<{}>", to_rust_type(inner)),
        GlossaType::Result(ok, err) => {
            format!("Result<{}, {}>", to_rust_type(ok), to_rust_type(err))
        }
        GlossaType::Unit => "()".to_string(),
        GlossaType::Struct { name, .. } => capitalize(&sanitize_name(name)),
        // TODO: Better representation for function types if they appear in type signatures
        GlossaType::Function { .. } => "fn".to_string(),
        GlossaType::Unknown => "_".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol_str::SmolStr;

    #[test]
    fn test_basic_types() {
        assert_eq!(to_rust_type(&GlossaType::Number), "i64");
        assert_eq!(to_rust_type(&GlossaType::String), "String");
        assert_eq!(to_rust_type(&GlossaType::Boolean), "bool");
        assert_eq!(to_rust_type(&GlossaType::Unit), "()");
        assert_eq!(to_rust_type(&GlossaType::Unknown), "_");
    }

    #[test]
    fn test_container_types() {
        assert_eq!(
            to_rust_type(&GlossaType::List(Box::new(GlossaType::Number))),
            "Vec<i64>"
        );
        assert_eq!(
            to_rust_type(&GlossaType::Set(Box::new(GlossaType::String))),
            "HashSet<String>"
        );
        assert_eq!(
            to_rust_type(&GlossaType::Map(
                Box::new(GlossaType::String),
                Box::new(GlossaType::Number)
            )),
            "HashMap<String, i64>"
        );
        assert_eq!(
            to_rust_type(&GlossaType::Option(Box::new(GlossaType::Number))),
            "Option<i64>"
        );
        assert_eq!(
            to_rust_type(&GlossaType::Result(
                Box::new(GlossaType::Number),
                Box::new(GlossaType::String)
            )),
            "Result<i64, String>"
        );
    }

    #[test]
    fn test_struct_type() {
        // Use normalized name as per compiler convention
        let ty = GlossaType::Struct {
            name: SmolStr::new("χρηστης"),
            gender: crate::morphology::Gender::Masculine,
            fields: vec![],
        };
        // Sanitize: χρηστης -> g__u3c7_rhsths
        // Capitalize: g__u3c7_rhsths -> G__u3c7_rhsths
        assert_eq!(to_rust_type(&ty), "G__u3c7_rhsths");
    }


    #[test]
    fn test_sanitize_greek_letter() {
        assert_eq!(sanitize_name("ξ"), "g_x");
        assert_eq!(sanitize_name("α"), "g_a");
        assert_eq!(sanitize_name("ω"), "g_w");
    }

    #[test]
    fn test_transliterate() {
        // χ (chi) -> hex, ρ -> r, η -> h, σ -> s, τ -> t, ο -> o, ς -> s
        // χ is 0x3c7
        assert_eq!(transliterate("χρηστος"), "_u3c7_rhstos");
        assert_eq!(transliterate("λογος"), "logos");
        // φ (phi) -> hex
        // φ is 0x3c6
        assert_eq!(transliterate("φιλοσοφια"), "_u3c6_iloso_u3c6_ia");
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
        assert_eq!(output, "a_u3df_b");
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
