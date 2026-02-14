//! Coverage tests for expression classification paths
//!
//! These tests target specific branches in `src/semantic/conversion.rs`
//! that handle standalone expressions (without verbs/assignments).

use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

fn compile_to_rust(source: &str) -> String {
    let ast = parse(source).expect("AST build failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    generate_rust(&analyzed)
}

#[test]
fn test_standalone_subject_op_object() {
    // "a" "b" greater (no verb) -> should be parsed as an expression
    // This hits the "Subject Op Object" branch in classify_expression
    let source = "
    α 1 ἔστω.
    β 2 ἔστω.
    α β μεῖζον.";

    let output = compile_to_rust(source);

    // Should compile to a comparison
    assert!(output.contains(">"), "Output should contain > operator");
    // Variables might be renamed, so we just check for the structure
    assert!(
        !output.contains("checked_"),
        "Comparison should not use checked math"
    );
}

#[test]
fn test_standalone_subject_op_literal() {
    // "a" 5 greater (no verb)
    // This hits the "Subject Op Literal" branch in classify_expression
    let source = "
    α 1 ἔστω.
    α 5 μεῖζον.";

    let output = compile_to_rust(source);

    assert!(output.contains(">"), "Output should contain > operator");
    assert!(output.contains("5"), "Output should contain literal 5");
}

#[test]
fn test_standalone_subject_op_nominative() {
    // "a" "b" greater (no verb) where "b" is parsed as a nominative (because Subject "a" is filled)
    // This hits the "Subject Op Nominative" branch in classify_expression fallback logic

    // In Glossa, if we have "Alpha Beta Greater.",
    // Alpha -> Subject (Nom)
    // Beta -> Nominative (Nom) - usually triggers "DoubleSubject" error if verb is present,
    // but without verb, Assembler might accept it if we feed carefully or if Assembler allows multiple nominatives.
    // Assembler::handle_nominal allows multiple nominatives if subject is filled.

    // Note: This source is identical to `test_standalone_subject_op_object` if β is parsed as object.
    // But `β` (Beta) is ambiguous. If we want it to be nominative, we rely on the parser/assembler logic.
    // To ensure it's Nominative, we might need a word that is explicitly Nominative.
    // Let's use `χρήστης` (Nom) vs `χρήστην` (Acc).

    let source = "
    χρήστης 1 ἔστω.
    α 2 ἔστω.
    α χρήστης μεῖζον.";

    let output = compile_to_rust(source);

    assert!(output.contains(">"), "Output should contain > operator");
}

#[test]
fn test_checked_arithmetic_codegen() {
    // Ensure checked_div is generated.
    // We use literals "10 2 μέρος" because the parser reliably converts the noun "μέρος"
    // to an operator when used with literals in this pattern.
    // This verifies the codegen path for checked division.
    let source = "10 2 μέρος λέγε.";

    let output = compile_to_rust(source);

    assert!(
        output.contains("checked_div"),
        "Should generate checked_div for division"
    );
}
