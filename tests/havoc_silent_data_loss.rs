use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_multiple_arrays_binding() {
    // "x 1 [1] [2] let."
    // x should bind to... what?
    // Arrays take priority.
    // So it binds to [1].
    // Ignores 1 and [2].
    // This confirms silent data loss.
    // We expect an Error (Ambiguous Value).
    let source = "ξ 1 [1] [2] ἔστω.";
    let ast = parse(source).expect("Failed to parse");
    let result = analyze_program(&ast);
    if result.is_ok() {
        println!("test_multiple_arrays_binding: Unexpected success: {:?}", result);
    } else {
        println!("test_multiple_arrays_binding: Expected error: {:?}", result);
    }
    assert!(
        result.is_err(),
        "Expected error for multiple array values, got Ok"
    );
}

#[test]
fn test_multiple_literals_binding() {
    // "x 1 2 let."
    let source = "ξ 1 2 ἔστω.";
    let ast = parse(source).expect("Failed to parse");
    let result = analyze_program(&ast);
    if result.is_ok() {
        println!("test_multiple_literals_binding: Unexpected success: {:?}", result);
    } else {
        println!("test_multiple_literals_binding: Expected error: {:?}", result);
    }
    assert!(
        result.is_err(),
        "Expected error for multiple literal values, got Ok"
    );
}

#[test]
fn test_mixed_binding() {
    // "x 1 [1] 2 let."
    let source = "ξ 1 [1] 2 ἔστω.";
    let ast = parse(source).expect("Failed to parse");
    let result = analyze_program(&ast);
    if result.is_ok() {
        println!("test_mixed_binding: Unexpected success: {:?}", result);
    } else {
        println!("test_mixed_binding: Expected error: {:?}", result);
    }
    assert!(
        result.is_err(),
        "Expected error for mixed array and literal values, got Ok"
    );
}
