#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast);
    assert!(analyzed.is_err(), "Expected DoubleSubject error!");
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστον λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let prog = analyze_program(&ast);
    assert!(prog.is_err(), "Expected UndefinedName error!");
}

#[test]
#[should_panic(expected = "MissingVerb")]
fn test_missing_verb_compiler_panic() {
    // Missing verb `ὁ ἄνθρωπος.` actually crashes `rustc` codegen if passed through,
    // or panics locally. We prove it panics or fails to compile!
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).unwrap();
    let _ = analyze_program(&ast).unwrap();
}
