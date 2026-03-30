use proptest::prelude::*;
use glossa::semantic::analyzer::analyze_program;
use glossa::parser::parse;

proptest! {
    #[test]
    fn test_does_not_crash(s in "\\PC*") {
        if let Ok(ast) = parse(&s) {
            let _ = analyze_program(&ast);
        }
    }
}
