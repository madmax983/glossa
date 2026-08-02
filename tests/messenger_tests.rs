#![cfg(feature = "nova")]

use glossa::tools::messenger::run_messenger;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_messenger_success() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test_schema.γλ");

    let source = "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. ἡλικία ἀριθμοῦ. }.";
    fs::write(&file_path, source).unwrap();

    let result = run_messenger(&file_path);
    assert!(
        result.is_ok(),
        "Messenger should succeed on valid struct definition"
    );
}

#[test]
fn test_messenger_file_not_found() {
    let path = PathBuf::from("does_not_exist.γλ");
    let result = run_messenger(&path);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Ἀρχεῖον οὐχ εὑρέθη")
    );
}

#[test]
fn test_messenger_syntax_error() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("bad_syntax.γλ");

    let source = "this is not valid glossa syntax";
    fs::write(&file_path, source).unwrap();

    let result = run_messenger(&file_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Σφάλμα"));
}
