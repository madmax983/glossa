#![cfg(feature = "nova")]

use glossa::tools::simulator::run_simulator;

#[test]
fn test_run_simulator_empty_output() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("empty_output.γλ");
    // Just evaluate an expression statement, which shouldn't print anything
    std::fs::write(&input_path, "1.").unwrap();

    let result = run_simulator(&input_path);
    assert!(result.is_ok());
}

#[test]
fn test_run_simulator_with_output() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("output.γλ");
    // Evaluate an expression and print it so output is not empty
    std::fs::write(&input_path, "«χαῖρε» λέγε.\n").unwrap();

    let result = run_simulator(&input_path);
    assert!(result.is_ok());
}

#[test]
fn test_run_simulator_print_multiple() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("print_multiple.γλ");
    // Just evaluate an expression statement, which shouldn't print anything
    std::fs::write(&input_path, "1 2 3 λέγε.").unwrap();

    let result = run_simulator(&input_path);
    assert!(result.is_ok());
}

#[test]
fn test_run_simulator_multiple_statements() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("multiple_stmts.γλ");
    std::fs::write(&input_path, "ξ 5 ἔστω.\nξ λέγε.\n").unwrap();

    let result = run_simulator(&input_path);
    assert!(result.is_ok());
}

#[test]
fn test_run_simulator_single_print() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("print_single.γλ");
    std::fs::write(&input_path, "1 λέγε.").unwrap();

    let result = run_simulator(&input_path);
    assert!(result.is_ok());
}

#[test]
fn test_run_simulator_multiple_stmts_and_outputs() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("multiple.γλ");
    std::fs::write(&input_path, "1 λέγε.\n2 λέγε.").unwrap();

    let result = run_simulator(&input_path);
    assert!(result.is_ok());
}

#[test]
fn test_run_simulator_multiple_errors() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("multiple_errors.γλ");
    std::fs::write(&input_path, "ξ 5 ἔστω.\n1 0 μέρος λέγε.").unwrap(); // binding then div by zero

    let result = run_simulator(&input_path);
    assert!(result.is_err());
}
