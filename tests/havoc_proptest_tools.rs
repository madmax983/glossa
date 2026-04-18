#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;
use glossa::tools::highlight::highlight;
use glossa::tools::narrator::tell_tale;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_highlight_fuzz(s in "\\PC*") {
        let _ = highlight(&s);
    }

    #[test]
    fn test_tell_tale_fuzz(s in "\\PC*") {
        if let Ok(ast) = parse(&s) && let Ok(analyzed) = analyze_program(&ast) {
                let _ = tell_tale(&analyzed);
            }
        }
}
