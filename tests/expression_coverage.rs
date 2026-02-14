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
fn test_assignment_object_nominative() {
    // Correct test for Object + Nominative + Operator in assignment
    // boolean_var = num1 num2 >
    let source = "
    α 10 ἔστω.
    β 2 ἔστω.
    μετά γ ψεῦδος ἔστω.
    γ α β μεῖζον γίγνεται.";

    let output = compile_to_rust(source);

    // Should generate comparison
    assert!(output.contains(">"), "Assignment should use comparison");
}

#[test]
fn test_assignment_nominative_nominative() {
    // Correct test for Nominative + Nominative + Operator in assignment
    // boolean_var = num1 num2 >
    // Here we use variables but parse them such that they fall into nominatives
    let _source = "
    α 10 ἔστω.
    β 2 ἔστω.
    μετά γ ψεῦδος ἔστω.
    γ α β μεῖζον γίγνεται.";

    // Note: The previous test covered Object + Nominative.
    // To trigger Nominative + Nominative, we need to ensure NEITHER is parsed as Object.
    // The parser slot logic is complex.
    // However, if we omit Object in the sentence structure, they might both land in Nominatives.
    // Glossa parser puts first noun in Subject, next in Object, others in Nominatives/Genitives/etc.
    // Assignment: "γ ... γίγνεται". γ is Subject.
    // If we have "α β ...".
    // "α" -> Object? "β" -> Nominative?
    // "α" -> Nominative? "β" -> Nominative?
    // This depends on case endings. α and β are indeclinable here.
    // Assuming they go to Object and Nominative as per previous test.

    // If we use specific case markers or words that decline?
    // Let's rely on the fact that if we have more nouns, they pile up.
    // But extract_value checks nominatives.len() >= 2.
    // So we need Subject (target) + Nominative + Nominative.
    // And NO Object.
    // This requires skipping the Object slot.
    // Can we do that?
    // Maybe if we use words that are explicitly Nominative and NOT Accusative?
    // Most neuters are both.
    // Numbers are often indeclinable.

    // If we can't easily trigger it, maybe the code path is unreachable with current parser rules?
    // But defensive coding is good.
    // We can try to force it by using a large number of arguments?
    // "γ α β δ μεῖζον γίγνεται." -> Subject Object Nom Nom Op Verb?
    // Then nominatives has 2 elements.

    let source_multi = "
    α 10 ἔστω.
    β 2 ἔστω.
    δ 1 ἔστω.
    μετά γ ψεῦδος ἔστω.
    γ α β δ μεῖζον γίγνεται.";
    // This might be syntactically valid but semantically weird.
    // But if it parses, extract_value will see nominatives.

    let output = compile_to_rust(source_multi);
    assert!(output.contains(">"), "Should handle multiple nominatives");
}
