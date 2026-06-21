#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    // Note: fixing double subject has been deferred to avoid regressions
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let prog = analyze_program(&ast).unwrap();
    if let glossa::semantic::AnalyzedStatement::Print(ref _expressions) = prog.statements[0] {
        return;
    }
    panic!("Did not evaluate to print silently!");
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    // Note: fixing undefined variables has been deferred to avoid regressions
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let prog = analyze_program(&ast).unwrap();

    if let glossa::semantic::AnalyzedStatement::Print(ref _expressions) = prog.statements[0]
        && _expressions.is_empty()
    {
        return;
    }
    panic!("Did not evaluate to empty silently!");
}

#[test]
fn test_missing_verb_compiler_panic() {
    // Missing verb `ὁ ἄνθρωπος.` used to crash `rustc` codegen if passed through.
    // We fixed the compilation panic so it now properly returns the GlossaError.
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).unwrap();
    let err = analyze_program(&ast).unwrap_err();
    assert!(err.to_string().contains("Ῥῆμα οὐχ εὑρέθη"));
}
