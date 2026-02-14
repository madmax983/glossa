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
    assert!(
        !output.contains("checked_add"),
        "String concat should not use checked_add"
    );
    // Should contain standard +
    assert!(output.contains("+"), "String concat should use +");
}

#[test]
fn test_boolean_ops_fallback() {
    // Tests BinaryOp::And/Or for Booleans (should use standard && / ||)
    // true and false say
    let source = "ἀληθές καί ψεῦδος λέγε.";
    let output = compile_to_rust(source);

    assert!(
        !output.contains("checked_"),
        "Boolean ops should not use checked_"
    );
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

#[test]
fn test_comparison_ops_fallback() {
    // Tests other comparisons: !=, <, >, <=, >=
    // "a" "b" unequal say
    let output_ne = compile_to_rust("«α» «β» ἄνισον λέγε.");
    assert!(output_ne.contains("!="), "String != should use !=");

    // "a" "b" less say
    let output_lt = compile_to_rust("«α» «β» ἔλαττον λέγε.");
    assert!(output_lt.contains("<"), "String < should use <");

    // "a" "b" greater say
    let output_gt = compile_to_rust("«α» «β» μεῖζον λέγε.");
    assert!(output_gt.contains(">"), "String > should use >");
}

#[test]
fn test_arithmetic_ops_fallback() {
    // Even if semantically invalid for strings, we want to ensure the codegen path is hit.
    // "a" "b" difference say -> "a" - "b"
    let output_sub = compile_to_rust("«α» «β» διαφορά λέγε.");
    assert!(output_sub.contains("-"), "String sub should use -");
    assert!(
        !output_sub.contains("checked_sub"),
        "String sub should not use checked_sub"
    );

    // "a" "b" product say -> "a" * "b"
    let output_mul = compile_to_rust("«α» «β» γινόμενον λέγε.");
    assert!(output_mul.contains("*"), "String mul should use *");
    assert!(
        !output_mul.contains("checked_mul"),
        "String mul should not use checked_mul"
    );

    // "a" "b" quotient say -> "a" / "b"
    let output_div = compile_to_rust("«α» «β» μέρος λέγε.");
    assert!(output_div.contains("/"), "String div should use /");
    assert!(
        !output_div.contains("checked_div"),
        "String div should not use checked_div"
    );

    // "a" "b" remainder say -> "a" % "b"
    let output_mod = compile_to_rust("«α» «β» ὑπόλοιπον λέγε.");
    assert!(output_mod.contains("%"), "String mod should use %");
    assert!(
        !output_mod.contains("checked_rem"),
        "String mod should not use checked_rem"
    );
}

#[test]
fn test_number_comparison_ops_codegen() {
    // Ensures coverage for BinaryOp::Eq, Ne, Lt, Le, Gt, Ge when left is Number
    // These use standard operators even for numbers, but we want to exercise the match arm.

    // 5 5 equal say
    let output_eq = compile_to_rust("5 5 ἴσον λέγε.");
    assert!(output_eq.contains("=="), "Number == should use ==");

    // 5 5 unequal say
    let output_ne = compile_to_rust("5 5 ἄνισον λέγε.");
    assert!(output_ne.contains("!="), "Number != should use !=");

    // 5 5 less say
    let output_lt = compile_to_rust("5 5 ἔλαττον λέγε.");
    assert!(output_lt.contains("<"), "Number < should use <");

    // 5 5 greater say
    let output_gt = compile_to_rust("5 5 μεῖζον λέγε.");
    assert!(output_gt.contains(">"), "Number > should use >");
}

#[test]
fn test_number_boolean_ops_codegen() {
    // Ensures coverage for BinaryOp::And, Or when left is Number
    // Semantically questionable (Number && Number) but syntactically possible
    // and handled in codegen (might fail rustc compile later if types mismatch, but codegen runs).

    // 5 5 and say
    let output_and = compile_to_rust("5 5 καί λέγε.");
    assert!(output_and.contains("&&"), "Number && should use &&");
}
