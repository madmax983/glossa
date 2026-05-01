use glossa::tools::tester::run_tests;
use tempfile::Builder;
use std::io::Write;

#[test]
fn test_print_test_results_coverage() {
    let mut temp_file = Builder::new()
        .prefix("glossa_test_")
        .suffix(".γλ")
        .tempfile()
        .unwrap();

    write!(
        temp_file,
        "δοκιμή «test_ok» {{ «ok» λέγε. }}.\nδοκιμή «test_fail» {{ assert_eq(1, 2). }}.\n"
    )
    .unwrap();

    let temp_path = temp_file.path().to_owned();

    // This will trigger print_test_results with mixed results (success and failure)
    let result = run_tests(&temp_path);
    assert!(result.is_err());
}
