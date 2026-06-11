#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_missing_verb_returns_semantic_error_instead_of_ice() {
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).expect("Parsing failed");

    let result = analyze_program(&ast);
    assert!(result.is_err(), "Expected an error due to missing verb");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Ἄγνωστον ὄνομα:"),
        "Expected UndefinedName error, got: {}",
        err_msg
    );
}
