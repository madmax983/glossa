use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_nested_phrase_binding_error() {
    let source = "α (1 (2 3)) ἔστω.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);

    assert!(result.is_err(), "Should error on nested phrase in binding");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Unexpected multiple terms"));
}
