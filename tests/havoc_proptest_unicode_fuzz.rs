use glossa::parser::parse;
use glossa::semantic::analyze_program;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_weird_unicode_fuzz(s in "\\PC*") {
        if let Ok(ast) = parse(&s) {
            let _ = analyze_program(&ast);
        }
    }
}
