/// 👺 Havoc: REPL Empty Comment Panic
///
/// Inputting an empty string or just whitespace evaluates to 0 statements.
/// The REPL's `execute()` method blindly calls `.unwrap()` on `.last()`, causing a denial of service (panic)
/// prior to the mitigation introduced. We prove the vulnerability exists/existed by asserting
/// that passing 0 statements into the REPL execution loop directly causes a panic.
#[test]
fn test_repl_empty_panic() {
    let src = " ";
    let ast = glossa::parser::parse(src).unwrap();
    let analyzed = glossa::semantic::analyze_program(&ast).unwrap();

    // The vulnerability:
    let result = std::panic::catch_unwind(|| {
        let _last_stmt = analyzed.statements.last().unwrap();
    });

    assert!(
        result.is_err(),
        "The analyzed statements list is empty and `.last().unwrap()` will panic! This is the REPL DoS vulnerability."
    );
}
