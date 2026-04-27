#![allow(missing_docs)]
use std::fs;
use std::process::Command;

#[test]
fn test_cli_fuzz() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("fuzz.γλ");

    // Write literal garbage
    fs::write(&file_path, "garb@ge i^nput #$$").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", "run", file_path.to_str().unwrap()])
        .output()
        .expect("failed to execute process");

    // As long as it exited (cleanly or with an error) but didn't crash via panic!
    // we consider it a success.
    // If it panics, it'll output "thread 'main' panicked"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("thread 'main' panicked"));
}
