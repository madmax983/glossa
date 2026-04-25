#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let res = analyze_program(&ast);
    assert!(res.is_err(), "Expected error for double subject: {:?}", res);
    let err = res.unwrap_err();
    assert!(matches!(err, glossa::errors::GlossaError::AssemblyError(glossa::errors::AssemblyError::DoubleSubject)), "Expected DoubleSubject, got {:?}", err);
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let res = analyze_program(&ast);
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), glossa::errors::GlossaError::UndefinedName { .. }));
}

#[test]
fn test_missing_verb_compiler_panic() {
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).unwrap();
    let res = analyze_program(&ast);
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), glossa::errors::GlossaError::AssemblyError(glossa::errors::AssemblyError::MissingVerb)));
}
