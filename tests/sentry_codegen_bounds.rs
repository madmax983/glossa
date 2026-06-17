use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_array_indexing_generates_panic_bounds_checks() {
    let source = "
    ξ [1, 2, 3] ἔστω.
    ξ[5] λέγε.
    ";
    let ast = parse(source).unwrap();
    let program = analyze_program(&ast).unwrap();
    let rust_code = generate_rust(&program).replace(" ", "");

    assert!(rust_code.contains("panic!(\"indexoutofbounds:negativeindex{}\",idx)"));
    assert!(rust_code.contains("expect(\"indexoutofbounds:toolarge\")"));
    assert!(rust_code.contains("expect(\"indexoutofbounds:indextoolarge\")"));
}
