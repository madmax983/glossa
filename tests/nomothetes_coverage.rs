use glossa::tools::nomothetes::run_nomothetes;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_run_nomothetes_success() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("test_nomo.γλ");

    fs::write(&input_path, "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.").unwrap();

    let result = run_nomothetes(&input_path);
    assert!(result.is_ok());
}

#[test]
fn test_run_nomothetes_file_not_found() {
    let path = PathBuf::from("non_existent_file.γλ");
    let result = run_nomothetes(&path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("οὐχ εὑρέθη"));
}

#[test]
fn test_run_nomothetes_empty_program() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("test_nomo_empty.γλ");

    fs::write(&input_path, "ξ 1 ἔστω.").unwrap();

    let result = run_nomothetes(&input_path);
    assert!(result.is_ok());
}

#[test]
fn test_run_nomothetes_parse_error() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("test_nomo_parse_error.γλ");
    fs::write(&input_path, "invalid syntax").unwrap();

    let result = run_nomothetes(&input_path);
    assert!(result.is_err());
}

#[test]
fn test_run_nomothetes_semantic_error() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("test_nomo_semantic_error.γλ");
    fs::write(&input_path, "ψ 10 γίγνεται.").unwrap();

    let result = run_nomothetes(&input_path);
    assert!(result.is_err());
}
