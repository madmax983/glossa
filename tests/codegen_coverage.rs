//! Coverage tests for code generation fallback paths
//!
//! These tests ensure that the fallback paths in `src/codegen/rust.rs`
//! (for non-numeric types) are fully exercised.

use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

fn compile_to_rust(source: &str) -> String {
    let ast = parse(source).expect("AST build failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    generate_rust(&analyzed)
}

#[test]
fn test_string_concatenation_fallback() {
    // Tests BinaryOp::Add for Strings (should use standard +)
    // "hello" " world" sum say
    let source = "«χαῖρε» « κόσμε» ἄθροισμα λέγε.";
    let output = compile_to_rust(source);

    // Should NOT contain checked_add
    assert!(!output.contains("checked_add"), "String concat should not use checked_add");
    // Should contain standard +
    assert!(output.contains("+"), "String concat should use +");
}

#[test]
fn test_boolean_ops_fallback() {
    // Tests BinaryOp::And/Or for Booleans (should use standard && / ||)
    // true and false say
    let source = "ἀληθές καί ψεῦδος λέγε.";
    let output = compile_to_rust(source);

    assert!(!output.contains("checked_"), "Boolean ops should not use checked_");
    assert!(output.contains("&&"), "Boolean AND should use &&");
}

#[test]
fn test_comparison_fallback() {
    // Tests comparison for non-numeric types (should use standard operators)
    // "a" "b" equal say
    let source = "«α» «β» ἴσον λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("=="), "String equality should use ==");
}
