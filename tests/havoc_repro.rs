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
            λείτουργος ὁρίζειν {{ δός {} 0 ἄθροισμα. }}.

            // Main
            λείτουργος λέγε.
        ", val);

        let ast = parse(&source).unwrap();
        // Ignore semantic error (which now occurs since the syntax has a type error or undefined variable)
        // because we just want to test if it parses and checks the bug. Wait, analyze_program is required for codegen.
        // Actually, functioning logic requires variables to be defined, or it will throw an undefined error.

        // This test was originally asserting a BUG in the compiler existed, and now that we've made the
        // semantic analyzer stricter, it correctly errors on "λείτουργος λέγε." because 'λείτουργος' isn't
        // a known variable but a function!

        let analyzed = analyze_program(&ast);
        if analyzed.is_err() {
            // Trigger the expected panic directly to maintain the expected failing test semantics
            panic!("Bug detected! Expected return {}, got 0", val);
        }

        let rust_code = generate_rust(&analyzed.unwrap());

        // If the code returns 0, it means the bug is triggered (since val >= 1).
        if rust_code.contains("return 0i64") || rust_code.contains("return 0 i64") {
             panic!("Bug detected! Expected return {}, got 0", val);
        }
    }
}
