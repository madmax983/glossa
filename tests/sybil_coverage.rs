#![cfg(feature = "nova")]

use glossa::tools::sybil::run_sybil;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_sybil_coverage() {
    let file = NamedTempFile::new().unwrap();
    let code = "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }. 10 λέγε.";
    fs::write(file.path(), code).unwrap();

    // Call run_sybil to cover the logic
    let res = run_sybil(file.path());
    assert!(res.is_ok());
}

#[test]
fn test_sybil_file_not_found() {
    let res = run_sybil(std::path::Path::new("nonexistent_file_12345.γλ"));
    assert!(res.is_err());
}

#[test]
fn test_sybil_parse_error() {
    let file = NamedTempFile::new().unwrap();
    let code = "εἶδος !@#$"; // invalid code
    fs::write(file.path(), code).unwrap();

    let res = run_sybil(file.path());
    assert!(res.is_err());
}
