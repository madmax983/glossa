#![allow(missing_docs)]
#![cfg(feature = "nova")]

use glossa::tools::labyrinth::{run_labyrinth, run_labyrinth_inner};

#[test]
fn test_run_labyrinth_comfy_table_output() {
    let source = "ξ 10 ἔστω.\n";
    let mut buffer = Vec::new();

    let result = run_labyrinth_inner(source, &mut buffer);
    assert!(result.is_ok());

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("Γ Λ Ω Σ Σ Α   L A B Y R I N T H"));
    assert!(output.contains("Mermaid.js Diagram"));
    assert!(output.contains("graph TD"));
    assert!(output.contains("Binding: ξ"));
    assert!(output.contains("Usage Instructions:"));
    assert!(output.contains("mermaid.live"));
}

#[test]
fn test_run_labyrinth_empty_output() {
    // Empty program to trigger the empty state branch
    let source = "";
    let mut buffer = Vec::new();

    let result = run_labyrinth_inner(source, &mut buffer);
    assert!(result.is_ok());

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("Γ Λ Ω Σ Σ Α   L A B Y R I N T H"));
    assert!(output.contains("Status"));
    assert!(output.contains("No control flow structures found."));
    // Should not contain the mermaid live instructions
    assert!(!output.contains("mermaid.live"));
}

#[test]
fn test_run_labyrinth_file_wrapper_success() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("test_labyrinth_wrapper.γλ");
    std::fs::write(&input_path, "ξ 10 ἔστω.\n").unwrap();

    let result = run_labyrinth(&input_path);
    assert!(result.is_ok());
}
