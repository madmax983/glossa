//! Unicode normalization for Ancient Greek text
//!
//! Converts polytonic Greek (with breathings, accents, iota subscript) to
//! a normalized monotonic form for easier processing.

use smol_str::SmolStr;
use std::char::ToLowercase;
use std::str::Chars;
use unicode_normalization::UnicodeNormalization;

/// Normalize polytonic Greek to monotonic form
///
/// This strips:
/// - Breathing marks (smooth ᾿ and rough ῾)
/// - Accents (acute ´, grave `, circumflex ῀)
/// - Iota subscript (ͅ)
/// - Diaeresis (¨)
///
/// And converts to lowercase for consistent comparison.
///
/// # Important: The Koronis
///
/// The Greek Koronis (᾽, U+1FBD), used in crasis (e.g., κἀγώ), is technically a
/// modifier letter, not a diacritic. `normalize_greek` **preserves** it.
/// This means `κἀγώ` normalizes to `κα᾽γω` (roughly), not `καγω`.
///
/// # Examples
///
/// ```
/// use glossa::text::normalize_greek;
///
/// assert_eq!(normalize_greek("ἄνθρωπος"), "ανθρωπος");
/// assert_eq!(normalize_greek("Ἀθῆναι"), "αθηναι");
/// assert_eq!(normalize_greek("χαῖρε"), "χαιρε");
///
/// // Koronis is preserved!
/// // Note: This string explicitly uses U+1FBD (Koronis), not the combining smooth breathing.
/// // Standard crasis like "κἀγώ" often uses combining marks which ARE stripped.
/// // But if you use the standalone character, it stays.
/// let text_with_koronis = "κ\u{1FBD}αγώ";
/// let normalized = normalize_greek(text_with_koronis);
///
/// assert!(normalized.contains('\u{1FBD}'));
/// assert_eq!(normalized, "κ\u{1FBD}αγω");
/// ```
pub fn normalize_greek(text: &str) -> SmolStr {
    let mut last_cased = false;

    // Optimization: Fast scan for characters that require normalization
    // This avoids all allocation for strings that are already normalized.
    for (i, c) in text.char_indices() {
        if c.is_uppercase() || !is_safe_char(c) {
            // Found a character that requires processing.
            // Split here: keep the safe prefix, process the rest.
            let mut output = String::with_capacity(text.len());
            output.push_str(&text[..i]);

            let suffix = &text[i..];
            let suffix_iter = GreekLowercaseIterator::new_with_state(suffix, last_cased)
                .nfd()
                .filter(|c| !is_greek_diacritic(*c));

            output.extend(suffix_iter);
            return output.into();
        }

        // Track casing for Sigma logic, just like the iterator would
        if !is_greek_diacritic(c) {
            last_cased = c.is_lowercase() || c.is_uppercase();
        }
    }

    // Tier 1: Text is already clean (lowercase, basic Greek/ASCII).
    // Zero-copy for small strings (inline), or single allocation for large.
    SmolStr::new(text)
}

/// Iterator that lowercases Greek text on the fly, correctly handling Sigma (σ/ς).
///
/// This iterator avoids allocating an intermediate `String` when lowercasing.
/// It implements the Sigma logic:
/// - Final Sigma (ς) if preceded by a cased letter and NOT followed by a cased letter.
/// - Medial Sigma (σ) otherwise.
struct GreekLowercaseIterator<'a> {
    iter: Chars<'a>,
    last_cased: bool,
    current_expansion: ToLowercase,
}

impl<'a> GreekLowercaseIterator<'a> {
    #[allow(dead_code)]
    fn new(text: &'a str) -> Self {
        Self::new_with_state(text, false)
    }

    fn new_with_state(text: &'a str, last_cased: bool) -> Self {
        let mut slf = Self {
            iter: text.chars(),
            last_cased,
            // Initialize with dummy that we drain immediately
            current_expansion: ' '.to_lowercase(),
        };
        slf.current_expansion.next();
        slf
    }
}

impl<'a> Iterator for GreekLowercaseIterator<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        // Yield chars from current expansion (e.g. if one char became multiple)
        if let Some(c) = self.current_expansion.next() {
            return Some(c);
        }

        // Fetch next char from input
        let c = self.iter.next()?;
        let mut char_to_process = c;

        if c == 'Σ' {
            if self.last_cased {
                let mut is_followed_by_cased = false;
                // Peek ahead in text skipping diacritics
                // Clone the iterator to look ahead without consuming the main iterator
                let lookahead = self.iter.clone();
                for next_c in lookahead {
                    if is_greek_diacritic(next_c) {
                        continue;
                    }
                    // Check if next char is cased
                    if next_c.is_lowercase() || next_c.is_uppercase() {
                        is_followed_by_cased = true;
                    }
                    break;
                }

                if !is_followed_by_cased {
                    char_to_process = 'ς';
                } else {
                    char_to_process = 'σ';
                }
            } else {
                char_to_process = 'σ';
            }
        }

        // Update last_cased for FUTURE iterations
        // Skip updating state if current char is a diacritic (Case_Ignorable)
        if !is_greek_diacritic(c) {
            self.last_cased = c.is_lowercase() || c.is_uppercase();
        }

        self.current_expansion = char_to_process.to_lowercase();
        self.current_expansion.next()
    }
}

/// Check if a character is "safe" (likely already normalized)
/// Safe characters are:
/// - ASCII characters (except uppercase letters)
/// - Basic Greek lowercase (α-ω)
/// - Final Sigma (ς)
///
/// Anything else (including diacritics, symbols like «, etc.) is considered "unsafe"
/// and triggers the decomposition path to ensure correctness.
#[inline]
fn is_safe_char(c: char) -> bool {
    if c.is_ascii() {
        !c.is_ascii_uppercase()
    } else {
        // Greek lowercase block: 0x03B1 (α) to 0x03C9 (ω)
        // Final sigma: 0x03C2 (ς)
        ('\u{03B1}'..='\u{03C9}').contains(&c) || c == '\u{03C2}'
    }
}

/// Check if a character is a Greek diacritical mark to be stripped
pub(crate) fn is_greek_diacritic(c: char) -> bool {
    matches!(
        c,
        '\u{0300}'..='\u{0304}'   // Combining grave, acute, circumflex, tilde, macron
        | '\u{0306}'              // Combining breve
        | '\u{0308}'              // Combining diaeresis
        | '\u{0313}'..='\u{0314}' // Combining smooth and rough breathing
        | '\u{0342}'..='\u{0345}' // Combining Greek perispomeni, koronis, dialytika tonos, ypogegrammeni
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_anthropos() {
        assert_eq!(normalize_greek("ἄνθρωπος"), "ανθρωπος");
    }

    #[test]
    fn test_normalize_athenai() {
        assert_eq!(normalize_greek("Ἀθῆναι"), "αθηναι");
    }

    #[test]
    fn test_normalize_chaire() {
        assert_eq!(normalize_greek("χαῖρε"), "χαιρε");
    }

    #[test]
    fn test_normalize_lege() {
        assert_eq!(normalize_greek("λέγε"), "λεγε");
    }

    #[test]
    fn test_normalize_esto() {
        assert_eq!(normalize_greek("ἔστω"), "εστω");
    }

    #[test]
    fn test_normalize_rough_breathing() {
        // Rough breathing on initial vowel
        assert_eq!(normalize_greek("ὁ"), "ο");
        assert_eq!(normalize_greek("ἡ"), "η");
    }

    #[test]
    fn test_normalize_iota_subscript() {
        // ᾳ = α + iota subscript
        assert_eq!(normalize_greek("τῇ"), "τη");
        assert_eq!(normalize_greek("τῷ"), "τω");
    }

    #[test]
    fn test_normalize_circumflex() {
        assert_eq!(normalize_greek("κῆπος"), "κηπος");
        assert_eq!(normalize_greek("δῶρον"), "δωρον");
    }

    #[test]
    fn test_normalize_preserves_base_letters() {
        // Should not change letters without diacritics
        assert_eq!(normalize_greek("κοσμος"), "κοσμος");
        assert_eq!(normalize_greek("λογος"), "λογος");
    }

    #[test]
    fn test_normalize_mixed_text() {
        // String with Greek and punctuation
        assert_eq!(normalize_greek("«χαῖρε κόσμε»"), "«χαιρε κοσμε»");
    }

    #[test]
    fn test_normalize_genitive_ending() {
        assert_eq!(normalize_greek("χρήστου"), "χρηστου");
        assert_eq!(normalize_greek("λόγου"), "λογου");
    }

    #[test]
    fn test_normalize_case_insensitive() {
        assert_eq!(normalize_greek("ΛΟΓΟΣ"), "λογος");
        assert_eq!(normalize_greek("Λόγος"), "λογος");
    }

    #[test]
    fn test_normalize_meizon() {
        // μεῖζον with circumflex should normalize to μειζον
        assert_eq!(normalize_greek("μεῖζον"), "μειζον");
    }

    #[test]
    fn test_normalize_or_particle() {
        // ἤ (eta with smooth breathing + acute) should normalize to η
        assert_eq!(normalize_greek("ἤ"), "η");
    }

    #[test]
    fn test_normalize_subjunctive_eimi() {
        // ᾖ (eta with iota subscript + circumflex) should normalize to η
        assert_eq!(normalize_greek("ᾖ"), "η");
    }

    #[test]
    fn test_normalize_sigma_cases() {
        // Test various Sigma cases using our zero-alloc iterator
        assert_eq!(normalize_greek("Σ"), "σ");
        assert_eq!(normalize_greek("Σ."), "σ.");
        assert_eq!(normalize_greek("AΣ"), "aς");
        assert_eq!(normalize_greek("AΣ."), "aς.");
        assert_eq!(normalize_greek("AΣA"), "aσa");
        assert_eq!(normalize_greek("1Σ"), "1σ");
        assert_eq!(normalize_greek("Σ́"), "σ"); // Diacritic stripped
    }

    #[test]
    fn test_normalize_diacritics_with_sigma() {
        // AΣ́ -> aς (A + Sigma + Acute).
        // Sigma preceded by A (cased).
        // Followed by Acute (diacritic, ignored) then EOF.
        // So Final Sigma.
        assert_eq!(normalize_greek("AΣ́"), "aς");
    }
}
#[cfg(test)]
mod tests_sentry_text_extra5 {
    use crate::text::{GreekLowercaseIterator, is_greek_diacritic, normalize_greek};

    #[test]
    fn test_text_normalization_early_return() {
        let input = "Αβγ";
        let output = normalize_greek(input);
        assert_eq!(output, "αβγ");
    }

    #[test]
    fn test_normalize_greek_safe_prefix_with_unsafe_suffix() {
        let input = "abcἄ";
        let output = normalize_greek(input);
        assert_eq!(output, "abcα");
    }

    #[test]
    fn test_greek_lowercase_iterator_direct() {
        let text = "Α";
        let iter = GreekLowercaseIterator::new(text);
        let result: String = iter.collect();
        assert_eq!(result, "α");
    }

    #[test]
    fn test_greek_lowercase_iterator_sigma_cased() {
        let text = "Σ";
        let iter = GreekLowercaseIterator::new_with_state(text, true);
        let result: String = iter.collect();
        assert_eq!(result, "ς");
    }

    #[test]
    fn test_greek_lowercase_iterator_sigma_cased_followed_by_cased() {
        let text = "Σα";
        let iter = GreekLowercaseIterator::new_with_state(text, true);
        let result: String = iter.collect();
        assert_eq!(result, "σα");
    }

    #[test]
    fn test_greek_lowercase_iterator_diacritic_followed_by_cased() {
        let text = "Σ\u{0301}α";
        let iter = GreekLowercaseIterator::new_with_state(text, true);
        let result: String = iter.collect();
        assert_eq!(result, "σ\u{0301}α");
    }

    #[test]
    fn test_is_greek_diacritic() {
        assert!(is_greek_diacritic('\u{0300}'));
        assert!(!is_greek_diacritic('α'));
    }
}
