use std::process::Command;

#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_glossa"))
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ΓΛΩΣΣΑ"));
    assert!(stdout.contains("Usage:"));
}

#[test]
fn test_cli_check() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("check.gl");
    std::fs::write(&file_path, "«test» λέγε.").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_glossa"))
        .arg("check")
        .arg(&file_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Check successful"));
}

#[test]
fn test_cli_run_direct() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("run_direct.gl");
    std::fs::write(&file_path, "«test» λέγε.").unwrap();

    // No subcommand, just file path
    let output = Command::new(env!("CARGO_BIN_EXE_glossa"))
        .arg(&file_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("test"));
}

#[test]
fn test_cli_run_subcommand() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("run_sub.gl");
    std::fs::write(&file_path, "«test» λέγε.").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_glossa"))
        .arg("run")
        .arg(&file_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("test"));
}

#[test]
fn test_cli_build() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("build.gl");
    std::fs::write(&file_path, "«test» λέγε.").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_glossa"))
        .arg("build")
        .arg(&file_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Build successful"));

    // Check if output file exists
    assert!(file_path.with_extension("rs").exists());
}

#[test]
fn test_cli_highlight() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("highlight.gl");
    std::fs::write(&file_path, "«test» λέγε.").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_glossa"))
        .arg("highlight")
        .arg(&file_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("test")); // Should verify highlighting ANSI codes maybe?
}

#[test]
fn test_cli_bard() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("bard.gl");
    std::fs::write(&file_path, "«test» λέγε.").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_glossa"))
        .arg("bard")
        .arg(&file_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Proclaim to the world"));
}
