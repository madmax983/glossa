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
fn test_operator_only_ignored() {
    // Test case where ONLY an operator exists (no subject, no literal, no object).
    // "Greater."
    // Left operand = None (no subject).
    // Right operand = None (no literals, no object, no nominatives).
    // Should fall through binary expression builder.

    let source = "μεῖζον.";

    let output = compile_to_rust(source);

    // Should NOT contain >
    assert!(
        !output.contains(">"),
        "Operator-only statement should not generate comparison"
    );
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
fn test_dangling_propagation() {
    // Test propagation (;) on a dangling expression.
    // "a" greater;
    // Build binary expr fails. Falls back to Subject "a".
    // Should generate "a?"

    let source = "
    α 1 ἔστω.
    α μεῖζον;";

    let output = compile_to_rust(source);

    assert!(!output.contains(">"), "Should not contain comparison");
    assert!(
        output.contains("?"),
        "Should contain try operator on the fallback variable"
    );
}

#[test]
fn test_excess_operators_ignored() {
    // Test case with more operators than operands can consume.
    // "10 2 sum difference" -> "10 2 + -"
    // Should consume +, but ignore -.
    // Expect: "10 + 2" (checked_add).

    let source = "10 2 ἄθροισμα διαφορά λέγε.";

    let output = compile_to_rust(source);

    assert!(output.contains("checked_add"), "Should contain checked_add");
    assert!(!output.contains("checked_sub"), "Should NOT contain checked_sub");
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
fn test_dangling_operator_ignored() {
    // Test case where an operator exists but operands are missing.
    // "a" greater. (Subject + Operator, no Right Operand)
    // Should fall through the binary expression builder and just emit the variable.

    let source = "
    α 1 ἔστω.
    α μεῖζον."; // "a greater."

    let output = compile_to_rust(source);

    // Should NOT contain > because the binary expression couldn't be built
    assert!(
        !output.contains(">"),
        "Dangling operator should be ignored/not generate invalid code"
    );
    // Should just print/emit 'a' (or whatever the default behavior is)
    // The default classify_expression falls back to build_expressions_from_literals_and_ops
    // If that's empty, it checks subject/object.
    // So it should just output 'a'.
}

#[test]
fn test_operator_without_subject_ignored() {
    // Test case where operator exists, literal exists, but no subject.
    // "5" greater. (Literal + Operator, no Subject)
    // Left operand logic tries to grab Subject. If missing -> None.
    // So binary expression build fails. Fallback -> Literal.

    let source = "5 μεῖζον.";

    let output = compile_to_rust(source);

    assert!(
        !output.contains(">"),
        "Operator without subject should be ignored"
    );
    assert!(output.contains("5"), "Should output the literal");
}
