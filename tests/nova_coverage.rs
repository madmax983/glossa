#![cfg(feature = "nova")]

use glossa::tools::tester::run_tests;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::Builder;

#[test]
fn test_run_weave_success() {
    let mut temp_file = Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "«χαῖρε κόσμε» λέγε.";
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let result = glossa::tools::weave::run_weave(temp_file.path());
    assert!(result.is_ok(), "Weave failed: {:?}", result.err());

    let output_path = temp_file.path().with_extension("md");
    assert!(output_path.exists());

    let md = fs::read_to_string(&output_path).unwrap();
    assert!(md.contains("# Rosetta Stone"));
    assert!(md.contains("```glossa"));
    assert!(md.contains("«χαῖρε κόσμε» λέγε."));
    assert!(md.contains("## 🧩 Semantic Assembly (Mosaic)"));
    assert!(md.contains("## 🦀 Generated Rust Code"));
    assert!(md.contains("```rust"));
    assert!(md.contains("println"));
}

#[test]
fn test_run_tests_no_tests_found() {
    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "ξ 1 ἔστω.";
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let result = run_tests(temp_file.path());
    assert!(result.is_ok(), "Test runner failed: {:?}", result.err());
}

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
fn test_run_simulate_command_error() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("simulate_error.gl");

    // Write valid file but simulate will fail due to semantic error (invalid division)
    std::fs::write(&input_path, "1 0 μέρος λέγε.").unwrap();

    let result = glossa::tools::interpreter::run_simulator(&input_path);
    assert!(result.is_err());
}

#[test]
fn test_run_simulate_command_not_nova() {
    // We cannot easily test the #[cfg(not(feature="nova"))] branch when we are compiling
    // with --all-features for coverage, but we can verify it fails gracefully if run as a subprocess
    // and compiled without the feature.

    // Instead of doing that, we will just rely on the other tests to cover the `simulate` feature
    // logic in `interpreter.rs` and accept the unhit lines in `main.rs` that only occur when the
    // feature is disabled. Actually, the `let _ = input; miette::bail!` block in `main.rs` is
    // only compiled when `nova` is NOT enabled. Since we run tests with `--all-features`, that code
    // is simply stripped out.
    // Let's test the success path of `main.rs` routing for simulate by spawning it, which will
    // improve `main.rs` branch coverage slightly.

    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("main_simulate.gl");
    std::fs::write(&input_path, "«test_main_simulate» λέγε.").unwrap();

    let bin_path = std::env::var("CARGO_BIN_EXE_glossa").unwrap_or_else(|_| {
        let llvm_cov_path = "target/llvm-cov-target/debug/glossa";
        if std::path::Path::new(llvm_cov_path).exists() {
            llvm_cov_path.to_string()
        } else {
            "target/debug/glossa".to_string()
        }
    });

    let mut cmd = std::process::Command::new(bin_path);
    let output = cmd.arg("simulate").arg(&input_path).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test_main_simulate"));
}

#[test]
fn test_run_simulate_command_success() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("simulate_success.gl");

    std::fs::write(&input_path, "«success» λέγε.").unwrap();

    let bin_path = std::env::var("CARGO_BIN_EXE_glossa").unwrap_or_else(|_| {
        let llvm_cov_path = "target/llvm-cov-target/debug/glossa";
        if std::path::Path::new(llvm_cov_path).exists() {
            llvm_cov_path.to_string()
        } else {
            "target/debug/glossa".to_string()
        }
    });

    let mut cmd = std::process::Command::new(bin_path);
    let output = cmd.arg("simulate").arg(&input_path).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("success"));
}
