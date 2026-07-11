#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let res = analyze_program(&ast);
    assert!(res.is_err());
    let err_str = res.unwrap_err().to_string();
    assert!(err_str.contains("Διπλοῦν ὑποκείμενον"));
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let res = analyze_program(&ast);
    assert!(
        res.is_err(),
        "Expected error for undefined variable, got ok!"
    );
    let err_str = res.unwrap_err().to_string();
    assert!(err_str.contains("Ἄγνωστον ὄνομα"));
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
