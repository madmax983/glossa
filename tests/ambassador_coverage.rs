#![allow(missing_docs)]
#![cfg(feature = "nova")]

use std::io::Write;
use tempfile::Builder;

#[test]
fn test_run_ambassador_success() {
    let mut temp_file = Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. ἡλικία ἀριθμοῦ. }.";
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let result = glossa::tools::ambassador::run_ambassador(temp_file.path());
    assert!(result.is_ok(), "Ambassador failed: {:?}", result.err());
}
