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
    let mut needs_decomposition = false;

    // Optimization: Fast scan for characters that require normalization
    // This avoids all allocation for strings that are already normalized.
    for c in text.chars() {
        if c.is_uppercase() {
            // Tier 2: Uppercase found.
            // We must use full normalization with lowercasing.
            // Using `str::to_lowercase` first ensures correct Sigma (Σ -> σ/ς) handling.
            return text
                .to_lowercase()
                .nfd()
                .filter(|c| !is_greek_diacritic(*c))
                .collect();
        }

        if !is_safe_char(c) {
            // Found a character that is not "safe" (e.g., has diacritics, or is a symbol).
            // Mark for decomposition but continue scanning for uppercase.
            needs_decomposition = true;
        }
    }

    if needs_decomposition {
        // Tier 3: No uppercase, but contains potential diacritics.
        // We can skip the `to_lowercase` allocation since we know it's already lower.
        return text.nfd().filter(|c| !is_greek_diacritic(*c)).collect();
    }

    // Tier 1: Text is already clean (lowercase, basic Greek/ASCII).
    // Zero-copy for small strings (inline), or single allocation for large.
    SmolStr::new(text)
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
}
