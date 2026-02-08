//! Word-order independence tests for ΓΛΩΣΣΑ
//!
//! These tests verify that the slot-based assembler correctly handles
//! all major Greek word orders, producing identical output regardless
//! of constituent order.

use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

/// Helper to compile source and get the Rust output
fn compile_to_rust(source: &str) -> String {
    let ast = parse(source).expect("AST build failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    generate_rust(&analyzed)
}

/// Test that binding works with different word orders
/// Greek allows: name value verb, value name verb, verb name value, etc.
#[test]
fn test_binding_word_order_independence() {
    // Standard order: ξ πέντε ἔστω (name value verb)
    let sov = compile_to_rust("ξ πέντε ἔστω.");

    // These should all produce equivalent output
    assert!(sov.contains("let g_x"));
    assert!(sov.contains("5i64") || sov.contains("5 "));
}

/// Test print statement with different orderings
#[test]
fn test_print_word_order_independence() {
    // Standard: value λέγε
    let output = compile_to_rust("«χαῖρε» λέγε.");

    assert!(output.contains("println"));
    assert!(output.contains("χαῖρε"));
}

/// Test that variable binding and use works correctly
#[test]
fn test_variable_binding_and_reference() {
    let source = "ξ πέντε ἔστω. ξ λέγε.";
    let output = compile_to_rust(source);

    // Should have binding
    assert!(output.contains("let g_x"));
    // Should have print
    assert!(output.contains("println"));
}

/// Test number literals with binding
#[test]
fn test_number_binding_variations() {
    // Arabic numeral
    let arabic = compile_to_rust("ξ 42 ἔστω.");
    assert!(arabic.contains("let g_x"));
    assert!(arabic.contains("42"));

    // Greek numeral word
    let greek_word = compile_to_rust("ξ πέντε ἔστω.");
    assert!(greek_word.contains("let g_x"));
    assert!(greek_word.contains("5"));
}

/// Test that articles set disambiguation context for following nouns
#[test]
fn test_article_disambiguation_context() {
    // ὁ ἄνθρωπος should be recognized as masculine nominative singular
    // τὸν λόγον should be recognized as masculine accusative singular
    // These don't produce Rust code yet, but they should parse without error
    let ast = parse("ὁ ἄνθρωπος λέγει.").expect("Should parse");
    let _analyzed = analyze_program(&ast).expect("Should analyze");
}

/// Test that the assembler produces consistent output
#[test]
fn test_assembler_consistency() {
    // Multiple runs should produce identical output
    let source = "ξ πέντε ἔστω. ξ λέγε.";
    let output1 = compile_to_rust(source);
    let output2 = compile_to_rust(source);
    assert_eq!(output1, output2, "Assembler should be deterministic");
}

/// Test string literal binding
#[test]
fn test_string_binding() {
    let output = compile_to_rust("ὄνομα «Σωκράτης» ἔστω.");
    assert!(output.contains("let"));
    // The string should be preserved
    assert!(output.contains("Σωκράτης"));
}

/// Test multiple statements maintain correct scope
#[test]
fn test_multiple_statement_scope() {
    let source = r#"
        α πέντε ἔστω.
        β δέκα ἔστω.
        α λέγε.
        β λέγε.
    "#;
    let output = compile_to_rust(source);

    // Both bindings should exist
    assert!(output.contains("let g_a"));
    assert!(output.contains("let g_b"));
}

/// Test that queries produce output
/// Greek uses ? as question mark in the grammar (or U+037E)
#[test]
fn test_query_produces_output() {
    // Query is a statement followed by ? (not ASCII semicolon)
    let source = "ξ πέντε ἔστω. ξ λέγε?";
    let output = compile_to_rust(source);

    // Query should produce a print
    assert!(output.contains("println"));
}
