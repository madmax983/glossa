use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).expect("Parsing failed");
    let analyzed = analyze_program(&ast);
    assert!(
        analyzed.is_err(),
        "Expected an error since double subject is now fixed"
    );
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε.";
    let ast = parse(source).expect("Parsing failed");
    let analyzed = analyze_program(&ast);
    assert!(
        analyzed.is_err(),
        "Expected an error since undefined variables are now fixed"
    );
}

#[test]
#[should_panic]
fn test_missing_verb_compiler_panic() {
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).expect("Parsing failed");

    // This now returns Err(MissingVerb), so unwrap() will panic, which is expected by #[should_panic]
    let _analyzed = analyze_program(&ast).unwrap();
}
