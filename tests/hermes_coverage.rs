#![cfg(feature = "nova")]

use glossa::tools::hermes::run_hermes;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_hermes_run_success() {
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("schema.γλ");

    // Replicating known-good syntax from papyrus tests
    let source = "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. ἡλικία ἀριθμοῦ. }. ξ 5 ἔστω.";
    fs::write(&input_path, source).unwrap();

    let result = run_hermes(&input_path);
    assert!(result.is_ok(), "Hermes failed: {:?}", result.err());
}

#[test]
fn test_hermes_file_not_found() {
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("missing.γλ");

    let result = run_hermes(&input_path);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Ἀρχεῖον οὐχ εὑρέθη"));
}

#[test]
fn test_hermes_invalid_syntax() {
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("invalid.γλ");

    let source = "this is not valid glossa";
    fs::write(&input_path, source).unwrap();

    let result = run_hermes(&input_path);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Σφάλμα ἀναλύσεως")
            || err_msg.contains("Parse error")
            || err_msg.contains("Expected")
    );
}
