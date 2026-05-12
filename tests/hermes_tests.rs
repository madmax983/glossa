#![cfg(feature = "nova")]

use std::io::Write;

#[test]
fn test_hermes_output() {
    let source = "
    εἶδος Χρήστης ὁρίζειν {
        ὄνομα ὀνόματος.
        ἡλικία ἀριθμοῦ.
        φίλοι λιστης.
    }.
    ";

    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    temp_file.write_all(source.as_bytes()).unwrap();

    // Since `run_hermes` prints to stdout and we want to capture its logic,
    // we could either refactor `run_hermes` to return the string or just execute the CLI
    // For this test, we'll execute the command line tool and capture stdout
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_glossa"))
        .arg("hermes")
        .arg(temp_file.path())
        .output()
        .expect("Failed to execute glossa hermes");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("export interface χρηστης {"));
    assert!(stdout.contains("ονομα: string;"));
    assert!(stdout.contains("ηλικια: number;"));
    assert!(stdout.contains("φιλοι: any[];"));
}
