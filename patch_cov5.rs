#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_extension() {
        // Test directories
        let dir_path = PathBuf::from("tests/test_data");
        assert!(validate_extension(&dir_path).is_ok());

        // Test valid files
        let valid_path1 = PathBuf::from("test.γλ");
        assert!(validate_extension(&valid_path1).is_ok());

        let valid_path2 = PathBuf::from("test.gl");
        assert!(validate_extension(&valid_path2).is_ok());

        // Test invalid files
        let invalid_path = PathBuf::from("test.md");
        let result = validate_extension(&invalid_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid file format: '.md'"));

        // Test missing extensions
        let missing_path = PathBuf::from("test");
        let result2 = validate_extension(&missing_path);
        assert!(result2.is_err());
        assert!(result2
            .unwrap_err()
            .to_string()
            .contains("Missing file extension"));
    }

    #[test]
    fn test_main_cli_args_missing_file_extension() {
        use std::process::Command;

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
    fn test_main_cli_args_invalid_file_extension() {
        use std::process::Command;

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
    }

    #[test]
    fn test_main_cli_args_global_invalid_file_extension() {
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
