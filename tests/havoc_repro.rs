#![allow(missing_docs)]
use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;
use proptest::prelude::*;

proptest! {
    // This test expects that return values are correctly generated.
    // However, due to a bug in `parse_return_expression`, complex expressions
    // are silently ignored and `0` is returned instead.
    // We expect this test to fail (panic) because the bug is present.
    #[test]
    #[should_panic(expected = "Bug detected!")]
    fn havoc_return_complex_expression(val in 1i64..1000) {
        // "δός <val> 0 ἄθροισμα." should return <val>.
        // But due to the bug, it returns 0.
        let source = format!("
            λειτουργος ὁρίζειν · δός {} 0 ἄθροισμα.

            // Main
            λειτουργος λέγε.
        ", val);

        let ast = parse(&source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let rust_code = generate_rust(&analyzed);

        // If the code returns 0, it means the bug is triggered (since val >= 1).
        if rust_code.contains("return 0i64") || rust_code.contains("return 0 i64") {
             panic!("Bug detected! Expected return {}, got 0", val);
        }
    }
}
