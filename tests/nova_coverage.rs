#![cfg(feature = "nova")]

use glossa::tools::tester::run_tests;
use std::io::Write;
use std::path::PathBuf;
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

#[test]
fn test_run_tests_file_not_found() {
    let path = PathBuf::from("non_existent_file.gl");
    let result = run_tests(&path);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Ἀρχεῖον οὐχ εὑρέθη")
    );
}

#[test]
fn test_run_tests_syntax_error() {
    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    write!(temp_file, "invalid syntax").expect("Failed to write");

    let result = run_tests(temp_file.path());
    assert!(result.is_err());
    // Error could be from parser or analyzer, but it should fail
}

#[test]
fn test_simulate_file_nova_coverage() {
    use glossa::tools::runner::simulate_file;

    // Test OK
    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    write!(temp_file, "ξ πέντε ἔστω.").expect("Failed to write");
    let result = simulate_file(temp_file.path());
    assert!(result.is_ok());

    // Test Analysis Error
    let mut temp_file2 = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    write!(temp_file2, "χαρακτήρ Χ ὁρίζειν ὡς Ψ.").expect("Failed to write");
    let result2 = simulate_file(temp_file2.path());
    assert!(result2.is_err());

    // Test Runtime Error
    let mut temp_file3 = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    write!(temp_file3, "1 0 μέρος λέγε.").expect("Failed to write");
    let result3 = simulate_file(temp_file3.path());
    assert!(result3.is_err());

    // Test file not found
    let result4 = simulate_file(PathBuf::from("non_existent_simulate_file.gl").as_path());
    assert!(result4.is_err());

    // Test Parse Error
    let mut temp_file5 = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    // Invalid syntax for parser (must not compile at all)
    write!(temp_file5, "+++++ invalid_syntax!!! +++").expect("Failed to write");
    let result5 = simulate_file(temp_file5.path());
    assert!(result5.is_err());
}

#[test]
fn test_simulate_file_coverage_runtime_err() {
    use glossa::tools::runner::simulate_file;

    // Test specific error paths
    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    write!(temp_file, "1 0 μέρος λέγε.").expect("Failed to write");
    let result = simulate_file(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_simulate_file_coverage_analysis_err() {
    use glossa::tools::runner::simulate_file;

    // Test specific error paths
    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    write!(temp_file, "χαρακτήρ Χ ὁρίζειν ὡς Ψ.").expect("Failed to write");
    let result = simulate_file(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_run_tests_rustc_error() {
    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    // This creates valid Glossa code that produces invalid Rust code (redefining String)
    // "εἶδος String ὁρίζειν { }." -> "struct String { }" -> conflicts with std::string::String
    let source = "εἶδος String ὁρίζειν { }. τέλος.";
    write!(temp_file, "{}", source).expect("Failed to write");

    let result = run_tests(temp_file.path());
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Rustc Error"));
}
