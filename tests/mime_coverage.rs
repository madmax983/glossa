#![cfg(feature = "nova")]

use glossa::tools::mime::run_mime;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_run_mime_file_not_found() {
    let result = run_mime(std::path::Path::new("does_not_exist_for_mime_coverage.γλ"));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Ἀρχεῖον οὐχ εὑρέθη"));
}

#[test]
fn test_run_mime_read_error() {
    let dir = tempdir().unwrap();
    let result = run_mime(dir.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Σφάλμα ἀρχείου"));
}

#[test]
fn test_run_mime_analyze_error() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("invalid.γλ");
    fs::write(&file_path, "invalid syntax").unwrap();
    let result = run_mime(&file_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Σφάλμα ἀναλύσεως"));
}

#[test]
fn test_run_mime_success() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("valid.γλ");
    fs::write(&file_path, "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. ἡλικία ἀριθμοῦ. }.").unwrap();
    let result = run_mime(&file_path);
    assert!(result.is_ok());
}
