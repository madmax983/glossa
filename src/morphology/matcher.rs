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
    fn test_match_suffix_greedy() {
        // Test early exit (false)
        let patterns = vec!["c", "bc", "abc"];
        // 1. "abc" ends with "c" -> stem "ab". Match. Stop.
        let mut matches = Vec::new();
        match_suffix("abc", &patterns, |p| p, |stem, pat| {
            matches.push((stem.to_string(), *pat));
            false // Stop after first match
        });
        assert_eq!(matches, vec![("ab".to_string(), "c")]);
    }

    #[test]
    fn test_match_suffix_exhaustive() {
        // Test continue (true)
        let patterns = vec!["c", "bc", "abc"];
        let mut matches = Vec::new();
        match_suffix("abc", &patterns, |p| p, |stem, pat| {
            matches.push((stem.to_string(), *pat));
            true // Continue
        });
        // 1. "c" -> stem "ab"
        // 2. "bc" -> stem "a"
        // 3. "abc" -> stem "" -> Should be skipped by !stem.is_empty() check
        assert_eq!(matches, vec![("ab".to_string(), "c"), ("a".to_string(), "bc")]);
    }

    #[test]
    fn test_match_suffix_empty_stem_ignored() {
        // Should verify that if the suffix equals the word, it's ignored
        let patterns = vec!["test"];
        let mut called = false;
        match_suffix("test", &patterns, |p| p, |_stem, _pat| {
            called = true;
            true
        });
        assert!(!called, "Should not match when stem is empty");
    }

    #[test]
    fn test_match_suffix_no_match() {
        let patterns = vec!["xyz"];
        let mut called = false;
        match_suffix("abc", &patterns, |p| p, |_stem, _pat| {
            called = true;
            true
        });
        assert!(!called);
    }
}
