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

#[test]
fn test_main_simulate_coverage() {
    use std::io::Write;
    use std::process::Command;
    use tempfile::Builder;

    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    write!(temp_file, "«hello» λέγε.").expect("Failed to write");

    // We can just run the binary with `simulate` to ensure that match arm in main.rs is covered.
    let output = Command::new(env!("CARGO_BIN_EXE_glossa"))
        .arg("simulate")
        .arg(temp_file.path())
        .output()
        .expect("Failed to execute glossa binary");

    assert!(output.status.success());
}

#[test]
fn test_cli_struct_simulate_branch() {
    // Tests that parsing the Simulate arg creates the correct Command enum
    use clap::Parser;
    use glossa::tools::cli::{Cli, Commands};

    let cli = Cli::parse_from(["glossa", "simulate", "test.gl"]);
    match cli.command {
        Some(Commands::Simulate { input }) => {
            assert_eq!(input.to_str().unwrap(), "test.gl");
        }
        _ => panic!("Expected Simulate command"),
    }
}

#[test]
fn test_main_simulate_coverage_error_branch() {
    use std::io::Write;
    use std::process::Command;
    use tempfile::Builder;

    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    write!(temp_file, "this is invalid").expect("Failed to write");

    let output = Command::new(env!("CARGO_BIN_EXE_glossa"))
        .arg("simulate")
        .arg(temp_file.path())
        .output()
        .expect("Failed to execute glossa binary");

    assert!(!output.status.success());
}

#[test]
fn test_main_simulate_with_invalid_arg() {
    use std::process::Command;
    // Call glossa simulate with no argument
    let output = Command::new(env!("CARGO_BIN_EXE_glossa"))
        .arg("simulate")
        .output()
        .expect("Failed to execute glossa binary");

    assert!(!output.status.success());
}
