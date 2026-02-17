#![cfg(feature = "nova")]

use glossa::tools::tester::run_tests;
use std::io::Write;
use tempfile::Builder;

#[test]
fn test_run_tests_success() {
    // Create a temporary Glossa file
    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    // Write a passing test
    // Test: Let x be 5. Assert x equals 5.
    let source = "
    δοκιμή «test_simple».
       ξ πέντε ἔστω.
       ξ πέντε ἰσοῦται.
    τέλος.
    ";

    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    // Run the tester
    let result = run_tests(temp_file.path());

    // Should succeed
    assert!(result.is_ok(), "Test runner failed: {:?}", result.err());
}

#[test]
fn test_run_tests_failure() {
    // Create a temporary Glossa file
    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    // Write a failing test
    // Test: Let x be 4. Assert x equals 5.
    let source = "
    δοκιμή «test_fail».
       ξ τέσσαρα ἔστω.
       ξ πέντε ἰσοῦται.
    τέλος.
    ";

    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    // Run the tester
    let result = run_tests(temp_file.path());

    // Should fail
    assert!(result.is_err(), "Test runner should have failed");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Tests failed"));
}
