//! Generic suffix matching utility for morphology
//!
//! This module provides a reusable function for matching suffixes against a word,
//! which is a common pattern in declension, conjugation, and participle analysis.

/// Match a word against a list of patterns by checking suffixes
///
/// Iterates through `patterns` and checks if `word` ends with the suffix provided by `get_suffix`.
/// If a match is found and the resulting stem is not empty, `callback` is invoked with the stem and the matching pattern.
///
/// The callback must return a `bool`:
/// - `true`: Continue searching for more matches.
/// - `false`: Stop searching (early exit).
///
/// # Type Parameters
///
/// * `T`: The type of the pattern element (e.g., tuple or struct)
/// * `S`: Closure that extracts the suffix string from `T`
/// * `F`: Callback closure invoked on match
pub fn match_suffix<'w, 'p, T, S, F>(
    word: &'w str,
    patterns: &'p [T],
    get_suffix: S,
    mut callback: F,
) where
    S: Fn(&T) -> &str,
    F: FnMut(&'w str, &'p T) -> bool,
{
    for pattern in patterns {
        let suffix = get_suffix(pattern);
        if let Some(stem) = word.strip_suffix(suffix) {
            // Stem must not be empty (avoid matching the entire word as a suffix if stem disappears)
            // e.g. "ω" matching "ω" ending -> stem "" -> usually invalid for our morphology rules
            if !stem.is_empty() && !callback(stem, pattern) {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_match() {
        let patterns = vec![("suffix", 1)];
        let mut matched = false;

        match_suffix(
            "wordsuffix",
            &patterns,
            |p| p.0,
            |stem, &(_, val)| {
                assert_eq!(stem, "word");
                assert_eq!(val, 1);
                matched = true;
                true
            },
        );

        assert!(matched, "Should have matched suffix");
    }

    #[test]
    fn test_multiple_matches() {
        // Matches should be found in order if callback returns true
        let patterns = vec![("ix", 1), ("fix", 2), ("suffix", 3)];
        // ⚡ Bolt Optimization: Uses `Vec::with_capacity` to prevent reallocation
        let mut matches = Vec::with_capacity(4);

        match_suffix(
            "suffix",
            &patterns,
            |p| p.0,
            |stem, &(_, val)| {
                matches.push((stem.to_string(), val));
                true // Continue searching
            },
        );

        // "suffix" ends with "ix" -> stem "suff", val 1
        // "suffix" ends with "fix" -> stem "suf", val 2
        // "suffix" ends with "suffix" -> stem "", val 3 -> SKIPPED (empty stem)

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], ("suff".to_string(), 1));
        assert_eq!(matches[1], ("suf".to_string(), 2));
    }

    #[test]
    fn test_early_exit() {
        let patterns = vec![("ix", 1), ("fix", 2)];
        // ⚡ Bolt Optimization: Uses `Vec::with_capacity` to prevent reallocation
        let mut matches = Vec::with_capacity(4);

        match_suffix(
            "suffix",
            &patterns,
            |p| p.0,
            |stem, &(_, val)| {
                matches.push((stem.to_string(), val));
                false // Stop searching after first match
            },
        );

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], ("suff".to_string(), 1));
    }

    #[test]
    fn test_empty_stem_check() {
        // This is a crucial constraint: match_suffix ignores matches where stem becomes empty
        let patterns = vec![("word", 1)];
        let mut matched = false;

        match_suffix(
            "word",
            &patterns,
            |p| p.0,
            |_, _| {
                matched = true;
                true
            },
        );

        assert!(!matched, "Should NOT match when stem is empty");
    }

    #[test]
    fn test_unicode_suffix() {
        // Test with Greek characters
        let patterns = vec![("ος", "nominative")];
        // ⚡ Bolt Optimization: Uses `Vec::with_capacity` to prevent reallocation
        let mut matches = Vec::with_capacity(4);

        match_suffix(
            "λογος",
            &patterns,
            |p| p.0,
            |stem, &(_, case)| {
                matches.push((stem.to_string(), case));
                true
            },
        );

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], ("λογ".to_string(), "nominative"));
    }

    #[test]
    fn test_no_match() {
        let patterns = vec![("a", 1), ("b", 2)];
        let mut matched = false;

        match_suffix(
            "c",
            &patterns,
            |p| p.0,
            |_, _| {
                matched = true;
                true
            },
        );

        assert!(!matched, "Should not match any pattern");
    }

    #[test]
    fn test_callback_mutable_state() {
        let patterns = vec![("a", 1), ("ba", 2)];
        let mut count = 0;

        match_suffix(
            "aba",
            &patterns,
            |p| p.0,
            |_, _| {
                count += 1;
                true
            },
        );

        // "aba" ends with "a" -> stem "ab" -> match
        // "aba" ends with "ba" -> stem "a" -> match
        assert_eq!(count, 2);
    }
}
