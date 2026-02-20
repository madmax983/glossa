use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_recursive_struct_fails() {
    // Define a recursive struct: struct A { a: A }
    // Syntax:
    // εἶδος Α ὁρίζειν {
    //     α Α.
    // }.
    let code = "εἶδος Α ὁρίζειν { α Α. }.";

    let ast = parse(code).expect("Failed to parse");
    let result = analyze_program(&ast);

    // Assert that semantic analysis fails
    assert!(
        result.is_err(),
        "Expected semantic analysis error for recursive struct, but got Ok"
    );

    let err = result.unwrap_err();
    let err_msg = err.to_string().to_lowercase();
    assert!(
        err_msg.contains("recursive"),
        "Expected error message to contain 'recursive', got: {}",
        err
    );
}
