#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let res = analyze_program(&ast);
    assert!(res.is_err(), "Double subject should fail");
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε.";
    let ast = parse(source).unwrap();
    let prog = analyze_program(&ast);
    assert!(
        prog.is_err(),
        "Should return an error instead of being silent!"
    );
}

#[test]
fn test_missing_verb_compiler_panic() {
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).unwrap();
    let res = analyze_program(&ast);
    assert!(res.is_err(), "Missing verb should fail");
}
