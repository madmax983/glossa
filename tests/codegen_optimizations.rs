use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_print_optimization_compile_time_format() {
    // Test that multi-argument print uses a compile-time format string
    // instead of runtime Vec concatenation.
    let source = "«α» «β» λέγε.";
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    let code = generate_rust(&analyzed);

    // Verify it uses the optimized format
    // Expected output roughly: println!("{} {}", "α", "β");

    // Check for the format string literal
    assert!(
        code.contains("\"{} {}\""),
        "Code should contain compile-time format string \"{{}} {{}}\". Code: {}",
        code
    );

    // Check absence of runtime allocation patterns
    // Previous inefficient code was: vec![...].join(" ")
    assert!(
        !code.contains("vec !"),
        "Code should not allocate Vec at runtime for print. Code: {}",
        code
    );
    assert!(
        !code.contains(". join"),
        "Code should not join strings at runtime for print. Code: {}",
        code
    );
}
