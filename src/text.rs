//! Unicode normalization for Ancient Greek text
//!
//! Converts polytonic Greek (with breathings, accents, iota subscript) to
//! a normalized monotonic form for easier processing.

use smol_str::SmolStr;
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
/// # Examples
///
/// ```
/// use glossa::text::normalize_greek;
///
/// assert_eq!(normalize_greek("ἄνθρωπος"), "ανθρωπος");
/// assert_eq!(normalize_greek("Ἀθῆναι"), "αθηναι");
/// assert_eq!(normalize_greek("χαῖρε"), "χαιρε");
/// ```
pub fn normalize_greek(text: &str) -> SmolStr {
    // Optimization: If the text contains uppercase characters, use the standard
    // `to_lowercase()` method which correctly handles Greek final sigma (ς).
    // This involves an extra allocation but ensures correctness.
    if text.chars().any(char::is_uppercase) {
        text.nfd()
            .filter(|c| !is_greek_diacritic(*c))
            .collect::<String>()
            .to_lowercase()
            .into()
    } else {
        // Fast path: If all characters are already lowercase (or non-cased),
        // we can avoid the intermediate allocation and complex casing logic.
        // We use a stack buffer for small strings to avoid heap allocation entirely.

        const STACK_BUF_SIZE: usize = 64;
        let mut buffer = [0u8; STACK_BUF_SIZE];
        let mut len = 0;
        let mut heap_buffer: Option<String> = None;

        for c in text.nfd() {
            if !is_greek_diacritic(c) {
                for lc in c.to_lowercase() {
                    let char_len = lc.len_utf8();

                    if let Some(ref mut s) = heap_buffer {
                        s.push(lc);
                    } else if len + char_len <= STACK_BUF_SIZE {
                        lc.encode_utf8(&mut buffer[len..]);
                        len += char_len;
                    } else {
                        // Overflow: switch to heap buffer
                        // Pre-allocate with some headroom (heuristic)
                        let mut s = String::with_capacity(text.len() + 16);
                        // Safe to unwrap because we constructed it from valid chars
                        s.push_str(std::str::from_utf8(&buffer[..len]).unwrap());
                        s.push(lc);
                        heap_buffer = Some(s);
                    }
                }
            }
        }

        if let Some(s) = heap_buffer {
            s.into()
        } else {
            // Safe to unwrap because we constructed it from valid chars
            SmolStr::new(std::str::from_utf8(&buffer[..len]).unwrap())
        }
    }
}

/// Check if a character is a Greek diacritical mark to be stripped
fn is_greek_diacritic(c: char) -> bool {
    matches!(
        c,
        '\u{0300}'          // Combining grave accent
        | '\u{0301}'        // Combining acute accent
        | '\u{0302}'        // Combining circumflex (hat)
        | '\u{0303}'        // Combining tilde
        | '\u{0304}'        // Combining macron
        | '\u{0306}'        // Combining breve
        | '\u{0308}'        // Combining diaeresis
        | '\u{0313}'        // Combining comma above (smooth breathing)
        | '\u{0314}'        // Combining reversed comma above (rough breathing)
        | '\u{0342}'        // Combining Greek perispomeni (circumflex)
        | '\u{0343}'        // Combining Greek koronis
        | '\u{0344}'        // Combining Greek dialytika tonos
        | '\u{0345}' // Combining Greek ypogegrammeni (iota subscript)
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
    fn test_normalize_long_string() {
        // Test fallback to heap allocation for strings > 64 bytes
        // "α" is 2 bytes in UTF-8. 40 alphas = 80 bytes > 64 bytes.
        let long_string = "α".repeat(40);
        assert_eq!(normalize_greek(&long_string), long_string.as_str());

        // With diacritics (should be stripped)
        // "ἀ" (alpha with smooth breathing) is U+1F00 (3 bytes) or U+03B1 U+0313 (2+2 bytes decomposed)
        // Let's use precomposed to test normalization
        let long_polytonic = "ἀ".repeat(40);
        let expected = "α".repeat(40);
        assert_eq!(normalize_greek(&long_polytonic), expected.as_str());
    }
}
