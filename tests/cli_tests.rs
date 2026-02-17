#[test]
fn test_cli_muse_execution() {
    // This test runs the actual binary to ensure src/main.rs code is covered
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_glossa"))
        .arg("muse")
        .arg("hero")
        .output()
        .expect("Failed to execute glossa binary");

    assert!(status.status.success());
    let stdout = String::from_utf8_lossy(&status.stdout);
    assert!(stdout.contains("Hero"));
    assert!(stdout.contains("ξ πέντε ἔστω"));
}
