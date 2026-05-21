use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    assert!(
        result.is_err(),
        "Undefined variable should result in an error"
    );
}

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ἔστω ἄνθρωπος 1. ἔστω θεός 2. ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    assert!(result.is_err(), "Double subject should result in an error");
}

#[test]
fn test_missing_verb_compiler_panic() {
    let source = "ἔστω ἄνθρωπος 1. ὁ ἄνθρωπος.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    assert!(result.is_err(), "Missing verb should result in an error");
}
