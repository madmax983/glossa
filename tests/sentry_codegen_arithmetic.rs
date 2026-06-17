use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_generate_checked_arithmetic_overflow_panic_checks() {
    let source = "
    ξ 5 ἔστω.
    ψ 3 ἔστω.
    ζ 5 ἄθροισμα 3 ἔστω.
    ";
    let ast = parse(source).unwrap();
    let program = analyze_program(&ast).unwrap();
    let rust_code = generate_rust(&program).replace(" ", "");
    assert!(rust_code.contains("checked_add"));
    assert!(rust_code.contains("expect(\"arithmeticoverflow\")"));
}
