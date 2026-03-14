use glossa::text::normalize_greek;
use proptest::prelude::*;

// Let's really hammer the GreekLowercaseIterator in `src/text.rs`
// specifically focusing on the lookahead logic for final sigma

proptest! {
    #[test]
    fn test_normalize_greek_fuzz_sigma(
        s in "(?i)(.*Σ.*){0,10}"
    ) {
        let _ = normalize_greek(&s);
    }
}
proptest! {
    #[test]
    fn test_normalize_greek_heavy_normalization(
        s in "(?i)(.*[\\u0300-\\u036F].*){0,10}"
    ) {
        let _ = normalize_greek(&s);
    }
}
proptest! {
    #[test]
    fn test_normalize_greek_massive_string(
        s in "(?i)(.*[\\u0300-\\u036F].*){0,200}"
    ) {
        let _ = normalize_greek(&s);
    }
}
