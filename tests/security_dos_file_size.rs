use std::fs::File;
use std::io::Write;
use std::process::Command;

#[test]
fn test_file_size_limit_cli() {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("large_dos.gl");

    // Create > 1MB file
    {
        let mut file = File::create(&file_path).expect("failed to create file");
        // 1MB + 1 byte
        let buffer = vec![b'a'; 1024 * 1024 + 1];
        file.write_all(&buffer).expect("failed to write file");
    }

    // Run glossa check
    // We expect this to fail
    let output = Command::new(env!("CARGO"))
        .arg("run")
        .arg("--quiet")
        .arg("--")
        .arg(&file_path)
        .arg("check")
        .output()
        .expect("failed to run glossa");

    // Clean up
    let _ = std::fs::remove_file(&file_path);

    if output.status.success() {
        panic!("Glossa should have failed on large file");
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Ἀρχεῖον λίαν μέγα") || stderr.contains("File too large"),
        "Expected error message about file size, got: {}",
        stderr
    );
}
