//! Unicode normalization for Ancient Greek text
//!
//! Converts polytonic Greek (with breathings, accents, iota subscript) to
//! a normalized monotonic form for easier processing.

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
/// use glossa::grammar::normalize_greek;
///
/// assert_eq!(normalize_greek("ἄνθρωπος"), "ανθρωπος");
/// assert_eq!(normalize_greek("Ἀθῆναι"), "αθηναι");
/// assert_eq!(normalize_greek("χαῖρε"), "χαιρε");
/// ```
pub fn normalize_greek(text: &str) -> String {
    text.nfd() // Decompose into base + combining marks
        .filter(|c| !is_greek_diacritic(*c))
        .collect::<String>()
        .to_lowercase()
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
        | '\u{0345}'        // Combining Greek ypogegrammeni (iota subscript)
    )
}

/// Normalize a single Greek word for lexicon lookup
#[allow(dead_code)]
pub fn normalize_word(word: &str) -> String {
    normalize_greek(word)
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
}
