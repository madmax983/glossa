use glossa::tools::tester::run_tests;
use std::fs;

#[test]
fn test_run_tests_integration() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test_passing.glossa");

    let source = "
δοκιμή «passing test».
    ξ 5 ἔστω.
    ξ 5 ἰσοῦται.
τέλος.
";
    fs::write(&file_path, source).unwrap();

    let result = run_tests(&file_path);
    assert!(result.is_ok());
}

#[test]
fn test_run_tests_failing_integration() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test_failing.glossa");

    let source = "
δοκιμή «failing test».
    ξ 5 ἔστω.
    ξ 4 ἰσοῦται.
τέλος.
";
    fs::write(&file_path, source).unwrap();

    let result = run_tests(&file_path);
    assert!(result.is_err());
}

#[test]
fn test_run_tests_empty_test_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test_empty.glossa");

    let source = "
ξ 5 ἔστω.
";
    fs::write(&file_path, source).unwrap();

    let result = run_tests(&file_path);
    assert!(result.is_ok());
}

#[test]
fn test_run_tests_invalid_input_path() {
    let result = run_tests(std::path::Path::new("non_existent_file.glossa"));
    assert!(result.is_err());
}

#[test]
fn test_run_tests_compilation_error() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test_compile_err.glossa");

    let source = "
δοκιμή «compile error».
    1 2 ἴσον ἔστω.
τέλος.
";
    fs::write(&file_path, source).unwrap();

    let result = run_tests(&file_path);
    assert!(result.is_err());
}
