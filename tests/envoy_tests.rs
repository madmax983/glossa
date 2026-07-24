#![cfg(feature = "nova")]

use glossa::tools::envoy::run_envoy;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_run_envoy_success() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.gl");
    let content = "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. ἡλικία ἀριθμοῦ. }.";
    fs::write(&file_path, content).unwrap();

    let result = run_envoy(&file_path);
    assert!(result.is_ok());
}

#[test]
fn test_run_envoy_file_not_found() {
    let result = run_envoy(std::path::Path::new("non_existent_file_for_envoy.gl"));
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Ἀρχεῖον οὐχ εὑρέθη")
    );
}

#[test]
fn test_run_envoy_parse_error() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_error.gl");
    let content = "invalid syntax";
    fs::write(&file_path, content).unwrap();

    let result = run_envoy(&file_path);
    assert!(result.is_err());
}

#[test]
fn test_run_envoy_semantic_error() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_semantic_error.gl");
    // valid syntax, but undefined identifier
    let content = "ξ πέντε γίγνεται.";
    fs::write(&file_path, content).unwrap();

    let result = run_envoy(&file_path);
    assert!(result.is_err());
}

#[test]
fn test_run_envoy_file_read_error() {
    let dir = tempdir().unwrap();
    // Directory paths exist but cannot be read as files, causing load_source to fail
    let dir_path = dir.path();
    let result = run_envoy(dir_path);
    assert!(result.is_err());
}
