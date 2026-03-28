use glossa::parser::parse;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_pest_parse_no_panic(s in "\\PC*") {
        let _ = parse(&s);
    }
}
