use std::process::Command;

#[test]
fn test_cli_invalid_extension() {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--bin")
        .arg("glossa")
        .arg("--")
        .arg("run")
        .arg("tests/test_data/non_existent.md");

    let output = cmd.output().expect("Failed to execute cargo run");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("Invalid file format: '.md'"));
    assert!(stderr.contains("source files must have a '.γλ' or '.gl' extension"));
}

#[test]
fn test_cli_missing_extension() {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--bin")
        .arg("glossa")
        .arg("--")
        .arg("run")
        .arg("tests/test_data/no_extension_file");

    let output = cmd.output().expect("Failed to execute cargo run");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("Missing file extension"));
}

#[test]
fn test_cli_dir_extension_bypass() {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--bin")
        .arg("glossa")
        .arg("--")
        .arg("run")
        .arg("tests/test_data"); // Just point it to a dir, checking it gets past validate

    let output = cmd.output().expect("Failed to execute cargo run");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    // Since it bypasses validate_extension, it will fail inside runner with an OS error instead of miette validation error
    assert!(!stderr.contains("Missing file extension"));
    assert!(!stderr.contains("Invalid file format"));
}
