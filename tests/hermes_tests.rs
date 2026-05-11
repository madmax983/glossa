#![cfg(feature = "nova")]
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_hermes_missing_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_glossa"))
        .args(["hermes", "missing_file_that_does_not_exist.glossa"])
        .output()
        .expect("Failed to execute glossa command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Ἀρχεῖον οὐχ εὑρέθη"));
}

#[test]
fn test_hermes_valid_struct() {
    let mut file = NamedTempFile::new().unwrap();
    let code = r#"
        εἶδος Χρήστης ὁρίζειν {
           ὄνομα ὀνόματος.
           ἡλικία ἀριθμοῦ.
        }.
    "#;
    file.write_all(code.as_bytes()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_glossa"))
        .args(["hermes", file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute glossa command");

    assert!(
        output.status.success(),
        "Hermes command failed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("export interface χρηστης {"));
    assert!(stdout.contains("ονομα: string;"));
    assert!(stdout.contains("ηλικια: number;"));
}
