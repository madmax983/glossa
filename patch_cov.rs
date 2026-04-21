#[cfg(test)]
mod additional_tests {
    use std::path::PathBuf;

    #[test]
    fn test_main_cli_args_missing_file_extension() {
        use std::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--bin")
            .arg("glossa")
            .arg("--")
            .arg("tests/test_data/no_extension_file");

        let output = cmd.output().expect("Failed to execute cargo run");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(!output.status.success());
        assert!(stderr.contains("Missing file extension"));
    }

    #[test]
    fn test_main_cli_args_invalid_file_extension() {
        use std::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--bin")
            .arg("glossa")
            .arg("--")
            .arg("tests/test_data/non_existent.md");

        let output = cmd.output().expect("Failed to execute cargo run");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(!output.status.success());
        assert!(stderr.contains("Invalid file format: '.md'"));
    }
}
