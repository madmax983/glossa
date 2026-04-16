#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    // Double subject should fail gracefully instead of compiling silently.
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Διπλοῦν ὑποκείμενον"));
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);

    // It should NOT evaluate to zero silently anymore. It should return an undefined error.
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Άγνωστον") || err_msg.contains("Ἄγνωστον"));
}

#[test]
fn test_missing_verb_compiler_panic() {
    // Missing verb `ὁ ἄνθρωπος.` used to crash rustc codegen or panic.
    // Now it safely returns the semantic/assembly MissingVerb error.
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Ῥῆμα οὐχ εὑρέθη"));
}
