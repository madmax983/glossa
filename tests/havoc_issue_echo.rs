#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let err = analyze_program(&ast).unwrap_err();
    assert!(matches!(
        err,
        glossa::errors::GlossaError::AssemblyError(glossa::errors::AssemblyError::DoubleSubject)
    ));
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let err = analyze_program(&ast).unwrap_err();
    assert!(matches!(
        err,
        glossa::errors::GlossaError::UndefinedName { .. }
    ));
}

#[test]
fn test_missing_verb_compiler_panic() {
    // The query without a verb triggers codegen failure.
    // Let's ensure it returns an UndefinedName instead, or MissingVerb!
    let source = "ὁ ἄνθρωπος;";
    let ast = parse(source).unwrap();
    let err = analyze_program(&ast).unwrap_err();
    assert!(matches!(
        err,
        glossa::errors::GlossaError::UndefinedName { .. }
    ));
}
