use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    #[test]
    fn test_parser_does_not_crash(s in ".*") {
        let _ = glossa::tools::runner::analyze_source(&s);
    }
}
