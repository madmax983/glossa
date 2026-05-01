use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_rejects_invalid_extensions() {
    let dir = tempdir().unwrap();

    // Create an invalid Rust file
    let rs_path = dir.path().join("test.rs");
    fs::write(&rs_path, "fn main() {}").unwrap();

    // Run the compiled glossa binary from target
    let bin_path = env!("CARGO_BIN_EXE_glossa");

    // Check multiple CLI commands to boost coverage
    let commands = [
        "run",
        "build",
        "check",
        "report",
        "highlight",
        "bard",
        "test",
        "mosaic",
        "map",
        "labyrinth",
        "weave",
        "alchemist",
        "papyrus",
        "haruspex",
        "audit",
        "gnomon",
        "scholar",
    ];

    for cmd in commands {
        let output_rs = Command::new(bin_path)
            .arg(cmd)
            .arg(&rs_path)
            .output()
            .expect("Failed to execute glossa binary");

        assert!(!output_rs.status.success());
        let stderr_rs = String::from_utf8_lossy(&output_rs.stderr);
        assert!(
            stderr_rs.contains("Invalid file extension"),
            "Failed for cmd: {}",
            cmd
        );
    }
}

#[test]
fn test_cli_accepts_valid_extensions() {
    let dir = tempdir().unwrap();

    // Create a valid Glossa file
    let gl_path = dir.path().join("test.gl");
    fs::write(&gl_path, "ξ 5 ἔστω.").unwrap();

    let bin_path = env!("CARGO_BIN_EXE_glossa");

    let output_gl = Command::new(bin_path)
        .arg(&gl_path)
        .output()
        .expect("Failed to execute glossa binary");

    let stderr_gl = String::from_utf8_lossy(&output_gl.stderr);
    assert!(!stderr_gl.contains("Invalid file extension"));
}

#[test]
fn test_cli_accepts_directories_as_valid_extensions() {
    let dir = tempdir().unwrap();

    // Test the directory itself as the input path
    let bin_path = env!("CARGO_BIN_EXE_glossa");

    let output_dir = Command::new(bin_path)
        .arg(dir.path())
        .output()
        .expect("Failed to execute glossa binary");

    let stderr_dir = String::from_utf8_lossy(&output_dir.stderr);
    assert!(!stderr_dir.contains("Invalid file extension"));
}
