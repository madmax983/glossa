#![allow(missing_docs)]
//! Operator integration tests for ΓΛΩΣΣΑ
//!
//! Tests for arithmetic, comparison, and boolean operators.
//! Following TDD: these tests are written BEFORE implementation.

use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

/// Helper to compile source and get the Rust output
fn compile_to_rust(source: &str) -> String {
    let ast = parse(source).expect("AST build failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    generate_rust(&analyzed)
}

// =============================================================================
// Comparison operator tests
// =============================================================================

#[test]
fn test_comparison_greater_than() {
    // μεῖζον means "greater" - should compile to >
    let source = "ξ πέντε ἔστω. ξ μεῖζον τριῶν λέγε.";
    let output = compile_to_rust(source);

    // Should contain the comparison operator
    assert!(output.contains(">"), "Expected > in output: {}", output);
}

#[test]
fn test_comparison_less_than() {
    // ἔλαττον means "lesser" - should compile to <
    let source = "ξ πέντε ἔστω. ξ ἔλαττον δέκα λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("<"), "Expected < in output: {}", output);
}

#[test]
fn test_comparison_equal() {
    // ἴσον means "equal" - should compile to ==
    let source = "ξ πέντε ἔστω. ξ ἴσον πέντε λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("=="), "Expected == in output: {}", output);
}

// =============================================================================
// Boolean operator tests
// =============================================================================

#[test]
fn test_boolean_and() {
    // καί means "and" - should compile to &&
    let source = "ἀληθές καί ἀληθές λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("&&"), "Expected && in output: {}", output);
}

#[test]
fn test_boolean_or() {
    // ἤ means "or" - should compile to ||
    let source = "ἀληθές ἤ ψεῦδος λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("||"), "Expected || in output: {}", output);
}

#[test]
fn test_boolean_not() {
    // οὐκ means "not" - should compile to !
    let source = "οὐκ ἀληθές λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("!"), "Expected ! in output: {}", output);
}

// =============================================================================
// Arithmetic operator tests
// =============================================================================

#[test]
fn test_arithmetic_sum() {
    // ἄθροισμα means "sum" - should compile to +
    let source = "πέντε τριῶν ἄθροισμα λέγε.";
    let output = compile_to_rust(source);

    assert!(
        output.contains("+") || output.contains("checked_add"),
        "Expected + in output: {}",
        output
    );
}

#[test]
fn test_arithmetic_difference() {
    // διαφορά means "difference" - should compile to -
    let source = "πέντε τριῶν διαφορά λέγε.";
    let output = compile_to_rust(source);

    assert!(
        output.contains("-") || output.contains("checked_sub"),
        "Expected - in output: {}",
        output
    );
}

#[test]
fn test_arithmetic_product() {
    // γινόμενον means "product" - should compile to *
    let source = "πέντε τριῶν γινόμενον λέγε.";
    let output = compile_to_rust(source);

    assert!(
        output.contains("*") || output.contains("checked_mul"),
        "Expected * in output: {}",
        output
    );
}

#[test]
fn test_arithmetic_quotient() {
    // μέρος means "quotient" - should compile to / (or checked_div)
    let source = "δέκα δύο μέρος λέγε.";
    let output = compile_to_rust(source);

    assert!(
        output.contains("/") || output.contains("checked_div"),
        "Expected / in output: {}",
        output
    );
}

#[test]
fn test_arithmetic_remainder() {
    // ὑπόλοιπον means "remainder" - should compile to % (or checked_rem)
    let source = "δέκα τριῶν ὑπόλοιπον λέγε.";
    let output = compile_to_rust(source);

    assert!(
        output.contains("%") || output.contains("checked_rem"),
        "Expected % in output: {}",
        output
    );
}

// =============================================================================
// Combined expression tests
// =============================================================================

#[test]
fn test_comparison_in_binding() {
    // Bind a comparison result to a variable
    let source = "ξ πέντε μεῖζον τριῶν ἔστω.";
    let output = compile_to_rust(source);

    assert!(output.contains("let"), "Expected let binding");
    assert!(output.contains(">"), "Expected > comparison");
}

#[test]
fn test_arithmetic_in_binding() {
    // Bind an arithmetic result to a variable
    let source = "ξ πέντε τριῶν ἄθροισμα ἔστω.";
    let output = compile_to_rust(source);

    assert!(output.contains("let"), "Expected let binding");
    assert!(
        output.contains("+") || output.contains("checked_add"),
        "Expected + operation"
    );
}
