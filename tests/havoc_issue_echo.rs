#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "
    ἄνθρωπος 1 ἔστω.
    θεὸς 2 ἔστω.
    ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Διπλοῦν ὑποκείμενον"));
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let prog = analyze_program(&ast);

    // Now it properly fails because undefined variable
    assert!(prog.is_err());
}

#[test]
fn test_missing_verb_compiler_panic() {
    let source = "
    ἄνθρωπος 1 ἔστω.
    ὁ ἄνθρωπος.
    ";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Ῥῆμα οὐχ εὑρέθη"));
}
