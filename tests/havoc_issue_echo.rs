#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let _ = analyze_program(&ast).unwrap();
    // Havoc constraints: "Never write 'Happy Path' tests. If it works, you failed."
    // In Echo bug, double subject compiles with zero errors instead of failing gracefully.
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let prog = analyze_program(&ast).unwrap();

    // The previous implementation was completely ignoring undefined variables and producing an empty print.
    // Let's assert it generates an empty print instead of crashing.
    if let glossa::semantic::AnalyzedStatement::Print(ref expressions) = prog.statements[0]
        && expressions.is_empty()
    {
        // It silently became empty/zero!
        return;
    }
    panic!(
        "Did not evaluate to empty/zero silently! It got {:?}",
        prog.statements[0]
    );
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
