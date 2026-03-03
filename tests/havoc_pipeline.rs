use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    // Fuzz the entire compilation pipeline
    // This catches panics in the analyzer or codegen if the parser happens to produce
    // a valid or seemingly-valid AST from garbage data.
    #[test]
    fn fuzz_entire_pipeline(s in "\\PC*") {
        if let Ok(ast) = parse(&s) {
            if let Ok(analyzed) = analyze_program(&ast) {
                // If it passes analysis, codegen MUST NOT PANIC!
                let _ = generate_rust(&analyzed);
            }
        }
    }

    // A more structured fuzzer targeting valid-looking syntax that might bypass initial parser rejections
    #[test]
    fn fuzz_pipeline_structured(s in "[a-zA-Z0-9_\\s\\.\\(\\)\\{\\}\\[\\]«»α-ωΑ-Ωἀ-ὥ]+") {
        if let Ok(ast) = parse(&s) {
            if let Ok(analyzed) = analyze_program(&ast) {
                let _ = generate_rust(&analyzed);
            }
        }
    }
}
