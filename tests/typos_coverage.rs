#![cfg(feature = "nova")]

use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use glossa::tools::typos::run_typos;

#[test]
fn test_typos_integration() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("schema.gl");
    let mut file = File::create(&file_path).unwrap();

    let source_code = "
    εἶδος Χρήστης ὁρίζειν {
        ὄνομα ὀνόματος.
        ἡλικία ἀριθμοῦ.
    }.
    ";

    file.write_all(source_code.as_bytes()).unwrap();

    let result = run_typos(&file_path);
    assert!(result.is_ok());
}
