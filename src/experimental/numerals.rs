//! Greek Numeral Parser
//!
//! Parses Ancient Greek alphabetic numerals into integers.
//!
//! # The System
//!
//! The Greek numeral system uses letters to represent numbers:
//! * 1-9: α, β, γ, δ, ε, ϛ, ζ, η, θ
//! * 10-90: ι, κ, λ, μ, ν, ξ, ο, π, ϟ
//! * 100-900: ρ, σ, τ, υ, φ, χ, ψ, ω, ϡ
//!
//! The "keraia" (ʹ, U+0374) marks a number (e.g., `αʹ` = 1).
//! The "lower keraia" (͵, U+0375) multiplies the following digit by 1000 (e.g., `͵α` = 1000).

use crate::text::normalize_greek;

/// Parse a Greek numeral string into an integer
///
/// Handles:
/// * Standard letters (α-ω)
/// * Archaic letters (stigma ϛ, koppa ϟ, sampi ϡ)
/// * Numeric markers (keraia ʹ, lower keraia ͵)
///
/// # Examples
///
/// ```
/// use glossa::experimental::numerals::parse_greek_numeral;
///
/// assert_eq!(parse_greek_numeral("αʹ").unwrap(), 1);
/// assert_eq!(parse_greek_numeral("βʹ").unwrap(), 2);
/// assert_eq!(parse_greek_numeral("ιαʹ").unwrap(), 11);
/// assert_eq!(parse_greek_numeral("ρʹ").unwrap(), 100);
/// assert_eq!(parse_greek_numeral("͵α").unwrap(), 1000);
/// assert_eq!(parse_greek_numeral("͵ααʹ").unwrap(), 1001);
/// ```
pub fn parse_greek_numeral(text: &str) -> Result<i64, String> {
    // Normalize first to handle case and diacritics
    // Note: normalize_greek lowercases everything
    let normalized = normalize_greek(text);

    let mut total: i64 = 0;
    let mut multiplier: i64 = 1;

    for c in normalized.chars() {
        match c {
            // Keraia (numeral sign) - ignore, it just marks the end or acts as punctuation
            // U+0374 (Dexia Keraia) or U+02B9 (Modifier Letter Prime) often used interchangeably
            // The literal 'ʹ' in source is usually one of these. We handle both explicitly.
            '\u{0374}' | '\u{02B9}' => continue,

            // Lower Keraia - multiplies the *next* digit by 1000
            '\u{0375}' => {
                multiplier = 1000;
                continue;
            }

            // Letters
            _ => {
                let value = match c {
                    'α' => 1,
                    'β' => 2,
                    'γ' => 3,
                    'δ' => 4,
                    'ε' => 5,
                    '\u{03DB}' | 'ς' => 6, // Stigma (03DB) or final sigma (03C2) fallback
                    'ζ' => 7,
                    'η' => 8,
                    'θ' => 9,
                    'ι' => 10,
                    'κ' => 20,
                    'λ' => 30,
                    'μ' => 40,
                    'ν' => 50,
                    'ξ' => 60,
                    'ο' => 70,
                    'π' => 80,
                    '\u{03D9}' | '\u{03DF}' => 90, // Koppa (archaic 03D9 / modern 03DF)
                    'ρ' => 100,
                    'σ' => 200,
                    'τ' => 300,
                    'υ' => 400,
                    'φ' => 500,
                    'χ' => 600,
                    'ψ' => 700,
                    'ω' => 800,
                    '\u{03E0}' | '\u{03E1}' => 900, // Sampi (03E0 / 03E1)
                    _ => return Err(format!("Invalid Greek numeral character: {} (U+{:04X})", c, c as u32)),
                };

                total += value * multiplier;
                // Reset multiplier after applying it to one digit
                multiplier = 1;
            }
        }
    }

    if total == 0 {
        return Err("Empty or invalid numeral".to_string());
    }

    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_units() {
        assert_eq!(parse_greek_numeral("αʹ").unwrap(), 1);
        assert_eq!(parse_greek_numeral("βʹ").unwrap(), 2);
        assert_eq!(parse_greek_numeral("θʹ").unwrap(), 9);
    }

    #[test]
    fn test_teens() {
        assert_eq!(parse_greek_numeral("ιαʹ").unwrap(), 11);
        assert_eq!(parse_greek_numeral("ιβʹ").unwrap(), 12);
        assert_eq!(parse_greek_numeral("ιθʹ").unwrap(), 19);
    }

    #[test]
    fn test_tens() {
        assert_eq!(parse_greek_numeral("κʹ").unwrap(), 20);
        assert_eq!(parse_greek_numeral("καʹ").unwrap(), 21);
        assert_eq!(parse_greek_numeral("λβʹ").unwrap(), 32); // 30 + 2
    }

    #[test]
    fn test_hundreds() {
        assert_eq!(parse_greek_numeral("ρʹ").unwrap(), 100);
        assert_eq!(parse_greek_numeral("σνγʹ").unwrap(), 253); // 200 + 50 + 3
    }

    #[test]
    fn test_thousands() {
        assert_eq!(parse_greek_numeral("͵α").unwrap(), 1000);
        assert_eq!(parse_greek_numeral("͵ααʹ").unwrap(), 1001);
        assert_eq!(parse_greek_numeral("͵β").unwrap(), 2000);
        assert_eq!(parse_greek_numeral("͵βκβʹ").unwrap(), 2022); // 2000 + 20 + 2
    }

    #[test]
    fn test_archaic() {
        // Using strict chars
        assert_eq!(parse_greek_numeral("\u{03DB}ʹ").unwrap(), 6); // Stigma
        assert_eq!(parse_greek_numeral("ςʹ").unwrap(), 6); // Final Sigma fallback

        // Koppa
        assert_eq!(parse_greek_numeral("\u{03DF}ʹ").unwrap(), 90);
        assert_eq!(parse_greek_numeral("\u{03D9}ʹ").unwrap(), 90);

        // Sampi
        assert_eq!(parse_greek_numeral("\u{03E1}ʹ").unwrap(), 900);
        assert_eq!(parse_greek_numeral("\u{03E0}ʹ").unwrap(), 900); // Sampi alt

        // Keraia alt (U+02B9)
        assert_eq!(parse_greek_numeral("α\u{02B9}").unwrap(), 1);
        // Dexia Keraia (U+0374)
        assert_eq!(parse_greek_numeral("α\u{0374}").unwrap(), 1);
    }

    #[test]
    fn test_invalid() {
        assert!(parse_greek_numeral("abc").is_err());
    }

    #[test]
    fn test_2024() {
        // 2000 = ͵β
        // 20 = κ
        // 4 = δ
        assert_eq!(parse_greek_numeral("͵βκδʹ").unwrap(), 2024);
    }

    #[test]
    fn test_full_coverage() {
        // Test every single character mapping to ensure 100% coverage
        let mappings = [
            ("α", 1), ("β", 2), ("γ", 3), ("δ", 4), ("ε", 5),
            ("\u{03DB}", 6), ("ς", 6), // Stigma
            ("ζ", 7), ("η", 8), ("θ", 9),
            ("ι", 10), ("κ", 20), ("λ", 30), ("μ", 40), ("ν", 50),
            ("ξ", 60), ("ο", 70), ("π", 80),
            ("\u{03D9}", 90), ("\u{03DF}", 90), // Koppa
            ("ρ", 100), ("σ", 200), ("τ", 300), ("υ", 400), ("φ", 500),
            ("χ", 600), ("ψ", 700), ("ω", 800),
            ("\u{03E0}", 900), ("\u{03E1}", 900) // Sampi
        ];

        for (char_str, expected) in mappings {
            // Test with standard keraia
            let input = format!("{}ʹ", char_str);
            assert_eq!(parse_greek_numeral(&input).unwrap(), expected, "Failed for {}", char_str);
        }
    }
}
