#[cfg(test)]
mod tests {
    use glossa::tools::runner::run_file;
    use std::io::Write;

    #[test]
    fn test_runner_recursion_error_propagation() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("recursion_error.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            // Trigger recursion limit (500)
            let deep_recursion = "(".repeat(600) + &")".repeat(600);
            f.write_all(deep_recursion.as_bytes()).unwrap();
        }

        let result = run_file(&input_path);

        if result.is_ok() {
            panic!("Expected error, but run_file succeeded!");
        }

        let err_msg = result.unwrap_err().to_string();

        // Expect Recursion Limit Error
        assert!(
            err_msg.contains("Recursion limit exceeded"),
            "Expected recursion error, got: {}",
            err_msg
        );
        assert!(
            !err_msg.contains("Codegen Failed"),
            "Should stop before codegen"
        );
    }

    #[test]
    fn test_runner_rustc_failure_output() {
        // This test replicates `test_run_rustc_error` from runner.rs but in a separate file
        // to ensure we can assert on the specific formatted output if needed.
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("rustc_fail.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            // Valid Glossa, invalid Rust (redefining String)
            f.write_all("εἶδος String ὁρίζειν { }. τέλος.".as_bytes())
                .unwrap();
        }

        let result = run_file(&input_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();

        assert!(err_msg.contains("Codegen Failed"));
        assert!(err_msg.contains("INTERNAL COMPILER ERROR"));
        // Check for visual elements from the new error formatting
        assert!(err_msg.contains("╔══"));
    }
}
