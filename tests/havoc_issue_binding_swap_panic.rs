#![allow(missing_docs)]

use glossa::parser::parse;
use glossa::semantic::analyze_program;
use std::env;
use std::process::Command;

#[test]
fn test_binding_swap_undefined_panic() {
    if env::var("HAVOC_TRIGGER_CRASH_SWAP").is_ok() {
        let code = r#"
            ξ 5 ἔστω.
            ξ ψ ἔστω.
        "#;
        let ast = parse(code).unwrap();
        let _ = analyze_program(&ast).unwrap();
        return;
    }

    let exe = env::current_exe().unwrap();
    let status = Command::new(exe)
        .arg("test_binding_swap_undefined_panic")
        .arg("--exact")
        .arg("--test-threads=1")
        .env("HAVOC_TRIGGER_CRASH_SWAP", "1")
        .status()
        .unwrap();

    assert!(!status.success(), "Process did not crash as expected!");
}
