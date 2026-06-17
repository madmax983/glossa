use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_generate_unwrap_panic() {
    let source = "
    ξ πέντε ἔστω.
    ξ! λέγε.
    ";
    let ast = parse(source).unwrap();
    let program = analyze_program(&ast).unwrap();
    let rust_code = generate_rust(&program).replace(" ", "");
    assert!(rust_code.contains("expect(\"attemptedtounwrapanemptyvalue\")"));
}
