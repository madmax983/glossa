use glossa::parser::parse;
use glossa::semantic::analyze_program;
use proptest::prelude::*;

proptest! {
    #[test]
    fn doesn_not_crash(s in "[αβγδεςζηθικλμνξοπϟρστυφχψωϡʹ͵λέγε\\s\\.\\(\\)\\{\\}\\[\\]«»]*") {
        if let Ok(ast) = parse(&s) {
            let _ = analyze_program(&ast);
        }
    }
}
