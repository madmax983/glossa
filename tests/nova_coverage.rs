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
fn test_timeline_tool_coverage() {
    let mut temp_file = tempfile::Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "μετά ξ 5 ἔστω. ξ 10 γίγνεται. εἰ ξ 5 μεῖζον ᾖ, «μεῖζον» λέγε. ἕως ξ 10 ἔλασσον ᾖ, ξ 10 γίγνεται. ἀριθμός [1, 2, 3] ἔστω. διὰ ἀριθμοῦ, ν λέγε.";
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let result = glossa::experimental::timeline::run_timeline(temp_file.path());
    assert!(result.is_ok(), "Timeline failed: {:?}", result.err());
}

#[test]
fn test_timeline_tool_file_not_found() {
    let result =
        glossa::experimental::timeline::run_timeline(&std::path::PathBuf::from("nonexistent.γλ"));
    assert!(result.is_err());
}
