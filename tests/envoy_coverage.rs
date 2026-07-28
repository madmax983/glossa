#![cfg(feature = "nova")]

use glossa::tools::envoy::run_envoy;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_run_envoy_success() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("schema.γλ");
    let mut file = std::fs::File::create(&file_path).unwrap();
    file.write_all("εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. ἡλικία ἀριθμοῦ. }.".as_bytes())
        .unwrap();

    let result = run_envoy(&file_path);
    assert!(result.is_ok());
}

#[test]
fn test_run_envoy_file_not_found() {
    let result = run_envoy(std::path::Path::new("non_existent_file.γλ"));
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
    let file_path = dir.path().join("invalid.γλ");
    let mut file = std::fs::File::create(&file_path).unwrap();
    file.write_all(b"invalid syntax!").unwrap();

    let result = run_envoy(&file_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Parse error"));
}

#[test]
fn test_run_envoy_semantic_error() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("semantic_error.γλ");
    let mut file = std::fs::File::create(&file_path).unwrap();
    file.write_all(b"\xce\xbb\xce\xb1\xce\xb8\xce\xbf\xce\xbf\xcf\x82 \xcf\x80\xce\xb5\xce\xbd\xcf\x84\xce\xb5 \xce\xb3\xce\xb9\xce\xb3\xce\xbd\xce\xb5\xcf\x84\xce\xb1\xce\xb9.").unwrap(); // "λαθοος πεντε γιγνεται."

    let result = run_envoy(&file_path);
    assert!(result.is_err());
}
