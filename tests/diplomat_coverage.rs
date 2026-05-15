#![cfg(feature = "nova")]

use glossa::experimental::diplomat::run_diplomat;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_diplomat_success() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("types.γλ");
    let mut file = File::create(&file_path).unwrap();

    // The syntax is: <Name> εἶδος { ... } ὁρίζειν. OR εἶδος <Name> { ... } ὁρίζειν.
    // Let's use the standard syntax
    let src = r#"
εἶδος Χρήστης ὁρίζειν {
    ὄνομα ὀνόματος.
    ἡλικία ἀριθμοῦ.
}.
"#;
    file.write_all(src.as_bytes()).unwrap();

    let result = run_diplomat(&file_path);
    assert!(
        result.is_ok(),
        "Diplomat failed on valid input: {:?}",
        result.err()
    );
}

#[test]
fn test_diplomat_file_not_found() {
    let result = run_diplomat(std::path::Path::new("does_not_exist.γλ"));
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Ἀρχεῖον οὐχ εὑρέθη"));
}

#[test]
fn test_diplomat_invalid_syntax() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("invalid.γλ");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"not a valid program").unwrap();

    let result = run_diplomat(&file_path);
    assert!(result.is_err());
}
