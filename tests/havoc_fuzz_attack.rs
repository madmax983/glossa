#![allow(missing_docs)]
use proptest::prelude::*;
use glossa::parser::parse;
use glossa::parser::parse_number_literal;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5000))]

    #[test]
    fn havoc_fuzz_parser_garbage(s in "\\PC*") {
        let _ = parse(&s);
    }

    #[test]
    fn havoc_fuzz_numerals_garbage(s in "\\PC*") {
        let _ = parse_number_literal(&s);
    }

    #[test]
    fn test_havoc_lexicon_fuzz(s in "\\PC*") {
        let _ = glossa::morphology::analyze(&s);
        let _ = glossa::morphology::analyze_all(&s);
    }
}
