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
            if !stem.is_empty() {
                if !callback(stem, pattern) {
                    return;
                }
            }
        }
    }
}
