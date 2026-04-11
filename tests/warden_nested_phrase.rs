#![allow(missing_docs)]
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

#[test]
fn test_function_definition_scope() {
    // Note: Added trailing period and return verb to avoid missing verb error
    let source = "συνάρτησις ὁρίζειν (χ)· { δὸς χ. }.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);

    if let Err(e) = &result {
        eprintln!("Func def error: {}", e);
    }
    let program = result.unwrap();
    assert!(
        program.scope.is_function("συναρτησις"),
        "Function should be in scope"
    );
}

#[test]
fn test_nested_phrase_valid_function() {
    // Define function 'myfunc' (foo)
    // Note: Added trailing period to first statement
    // Use `α (foo 2) ἔστω.` instead of `α (1 (foo 2)) ἔστω.`
    let source = "
    foo ὁρίζειν (x)· { x. }.
    α (foo 2) ἔστω.
    ";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);

    if let Err(e) = &result {
        eprintln!("Analysis error: {}", e);
    }
    assert!(
        result.is_ok(),
        "Should accept valid nested function call inside binding"
    );
}
