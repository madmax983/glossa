#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "Bug detected!")]
    fn havoc_return_bug_found(val in 1i64..1000) {
        let source = format!("
            λείτουργος ὁρίζειν · δός {} 0 ἄθροισμα.
            λείτουργος λέγε.
        ", val);
        let ast = parse(&source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let rust_code = glossa::codegen::generate_rust(&analyzed);
        if rust_code.contains("return 0i64") || rust_code.contains("return 0 i64") {
            panic!("Bug detected! Expected {}, got 0", val);
        }
    }
}
