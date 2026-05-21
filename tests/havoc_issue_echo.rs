#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;
use glossa::errors::GlossaError;
use glossa::errors::AssemblyError;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let err = analyze_program(&ast).unwrap_err();
    assert!(matches!(err, GlossaError::AssemblyError(AssemblyError::DoubleSubject)));
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε.";
    let ast = parse(source).unwrap();
    let err = analyze_program(&ast).unwrap_err();
    assert!(matches!(err, GlossaError::UndefinedName { .. }));
}

#[test]
fn test_missing_verb_compiler_panic() {
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).unwrap();
    let err = analyze_program(&ast).unwrap_err();
    assert!(matches!(err, GlossaError::AssemblyError(AssemblyError::MissingVerb)));
}
