use glossa::parser::numerals::parse_greek_numeral;
use glossa::parser::parse;
use glossa::text::normalize_greek;
use proptest::prelude::*;

proptest! {
    // Fuzz the parser with arbitrary strings
    #[test]
    fn fuzz_parser(s in "\\PC*") {
        // We expect errors for garbage input, but NOT panics
        let _ = parse(&s);
    }

    // Fuzz normalization with arbitrary strings
    #[test]
    fn fuzz_normalization(s in "\\PC*") {
        let _ = normalize_greek(&s);
    }

    // Fuzz numeral parser with arbitrary strings
    #[test]
    fn fuzz_numerals(s in "\\PC*") {
        let _ = parse_greek_numeral(&s);
    }
}
