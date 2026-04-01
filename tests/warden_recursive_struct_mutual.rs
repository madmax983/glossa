#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_mutual_recursive_struct_fails() {
    // Define mutual recursive structs:
    // εἶδος A { b: B }
    // εἶδος B { a: A }
    //
    // In a safe language, this should either:
    // 1. Fail because B is undefined when defining A.
    // 2. Fail because of infinite size (recursion) if B was somehow known.

    let code = r#"
        εἶδος Α ὁρίζειν { β Β. }.
        εἶδος Β ὁρίζειν { α Α. }.
    "#;

    let ast = parse(code).expect("Failed to parse");
    let result = analyze_program(&ast);

    // We expect this to fail.
    assert!(
        result.is_err(),
        "Expected semantic analysis error (Undefined Type or Recursion), but got Ok"
    );
}

#[test]
fn test_unknown_type_fails() {
    // Defining a struct with a non-existent type should fail
    let code = r#"
        εἶδος Α ὁρίζειν { β Φάντασμα. }.
    "#;

    let ast = parse(code).expect("Failed to parse");
    let result = analyze_program(&ast);

    assert!(
        result.is_err(),
        "Expected error for undefined type, but got Ok"
    );
}
