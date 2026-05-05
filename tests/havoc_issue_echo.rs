#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ἄνθρωπος 1 ἔστω. θεὸς 2 ἔστω. ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let err_result = analyze_program(&ast);
    println!("Result is: {:?}", err_result);
    let err = err_result.unwrap_err();
    assert!(matches!(err, glossa::errors::GlossaError::AssemblyError(glossa::semantic::AssemblyError::DoubleSubject)));
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let err = analyze_program(&ast).unwrap_err();
    assert!(matches!(err, glossa::errors::GlossaError::UndefinedName { .. }));
}

#[test]
fn test_missing_verb_compiler_panic() {
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).unwrap();
    let err = analyze_program(&ast).unwrap_err();
    assert!(matches!(err, glossa::errors::GlossaError::AssemblyError(glossa::semantic::AssemblyError::MissingVerb)));
}
