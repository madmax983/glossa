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
#[should_panic(expected = "Operator without operands")]
fn test_operator_only_ignored() {
    // Test case where ONLY an operator exists (no subject, no literal, no object).
    // "Equal."
    // Should now return an error instead of silently ignoring the operator.

    let source = "ἴσον.";

    compile_to_rust(source);
}

#[test]
fn test_expression_propagation() {
    // Test propagation (;) on a binary expression.
    // "a" "b" greater;
    // Should generate a Try operator (?) on the result.

    let source = "
    α 1 ἔστω.
    β 2 ἔστω.
    α β μεῖζον;";

    let output = compile_to_rust(source);

    assert!(output.contains(">"), "Should contain comparison");
    assert!(output.contains("?"), "Should contain try operator");
}

#[test]
#[should_panic(expected = "Operator without operands")]
fn test_dangling_propagation() {
    // Test propagation (;) on a dangling expression.
    // "a" equal;
    // Should return error because operator is unused.

    let source = "
    α 1 ἔστω.
    α ἴσον;";

    compile_to_rust(source);
}

#[test]
#[should_panic(expected = "Insufficient literals")]
fn test_excess_operators_error() {
    // Test case with more operators than operands can consume.
    // "10 2 sum difference" -> "10 2 + -"
    // Previously ignored, now strictly validated.
    // Should fail because we have 2 literals and 2 operators (need 3 literals).

    let source = "10 2 ἄθροισμα διαφορά λέγε.";

    compile_to_rust(source);
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

#[test]
#[should_panic(expected = "Operator without operands")]
fn test_dangling_operator_ignored() {
    // Test case where an operator exists but operands are missing.
    // "a" equal. (Subject + Operator, no Right Operand)
    // Should return error.

    let source = "
    α 1 ἔστω.
    α ἴσον."; // "a equal."

    compile_to_rust(source);
}

#[test]
#[should_panic(expected = "Operator without operands")]
fn test_operator_without_subject_ignored() {
    // Test case where operator exists, literal exists, but no subject.
    // "5" equal. (Literal + Operator, no Subject)
    // Should return error.

    let source = "5 ἴσον.";

    compile_to_rust(source);
}
