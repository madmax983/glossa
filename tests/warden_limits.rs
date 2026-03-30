use glossa::tools::run_file;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_operator_limit() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("operators.gl");

    // Create a chain of additions: 1 αθροισμα 1 αθροισμα ... (257 times)
    // 257 operators require 258 literals.
    let limit = 256;
    let mut code = "1 ".to_string();
    for _ in 0..(limit + 1) {
        code.push_str("αθροισμα 1 ");
    }
    code.push_str("λέγε.");

    let mut f = File::create(&file_path).unwrap();
    f.write_all(code.as_bytes()).unwrap();

    let result = run_file(&file_path);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Operators"));
    assert!(err.contains("256"));
}

#[test]
fn test_recursion_limit() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("recursion.gl");

    // 501 nested parentheses
    let depth = 501;
    let mut code = String::new();
    for _ in 0..depth {
        code.push('(');
    }
    code.push('1');
    for _ in 0..depth {
        code.push(')');
    }
    code.push_str(" λέγε.");

    let mut f = File::create(&file_path).unwrap();
    f.write_all(code.as_bytes()).unwrap();

    let result = run_file(&file_path);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Recursion limit exceeded"));
    assert!(err.contains("250"));
}
