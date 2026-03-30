use glossa::tools::tester::run_tests;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_run_tests_rustc_compilation_error() {
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("test_rustc_error.gl");
    // Invalid Rust code generation (e.g. invalid struct definition output)
    fs::write(&input_path, "εἶδος String ὁρίζειν { }. τέλος. δοκιμή «test» { }.").unwrap();

    let res = run_tests(&input_path);

    assert!(res.is_err());
    let err_str = res.unwrap_err().to_string();
    assert!(err_str.contains("Test Compilation Error") || err_str.contains("Rustc Error:"));
}
