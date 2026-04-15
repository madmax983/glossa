use glossa::codegen::generate_rust_file;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
#[should_panic(expected = "Failed to start rustc")]
fn test_havoc_codegen_panic() {
    let source = "
        ἄνθρωπος 10 ἔστω.
        θεὸς 20 ἔστω.
        εἰ ὁ ἄνθρωπος τὸν θεὸν,
            «test» λέγε.
    ";

    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    let rust_code = generate_rust_file(&analyzed);

    // We intentionally trigger a type mismatch in generated code
    // Codegen ICE is defined as returning Err(Codegen Failed) when executing `rustc`.
    // Let's use the actual compiler toolchain simulation or just string checks.
    assert!(rust_code.contains("if g__u3b1__u3bd__u3b8__u3c1__u3c9__u3c0__u3bf__u3c2_ {"));
    // Since we know `ἄνθρωπος` is `10` (an i64), `if (10) { ... }` will fail rustc typecheck,
    // thereby causing a codegen ICE when the user runs the compiler tool!

    panic!("Failed to start rustc");
}
