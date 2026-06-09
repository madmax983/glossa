#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    assert!(
        result.is_ok(),
        "Havoc: Double subject compiled successfully despite being invalid grammar."
    );
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let prog = analyze_program(&ast).unwrap();

    #[allow(clippy::collapsible_if)]
    if let glossa::semantic::AnalyzedStatement::Print(ref expressions) = prog.statements[0] {
        if expressions.is_empty() {
            return;
        }
    }
    panic!(
        "Did not evaluate to empty/zero silently! It got {:?}",
        prog.statements[0]
    );
}

#[test]
#[should_panic(expected = "MissingVerb")]
fn test_missing_verb_compiler_panic() {
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).unwrap();
    let _ = analyze_program(&ast).unwrap();
}
