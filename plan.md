1. **Add test for newly extracted functions in `src/tools/tester.rs`**:
   - Use `replace_with_git_merge_diff` to add tests for `print_summary_table`, `print_results_table`, `print_failure_details`, and `print_test_results` in `src/tools/tester.rs` by inserting:
```
<<<<<<< SEARCH
    #[test]
    fn test_run_tests_semantic_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("semantic_error.gl");
        std::fs::write(&input_path, "ψ 10 γίγνεται.").unwrap();

        let result = run_tests(&input_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // The underlying error bubbles up.
        assert!(err_msg.contains("Semantic error") || err_msg.contains("Σφάλμα"));
    }
}
=======
    #[test]
    fn test_run_tests_semantic_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("semantic_error.gl");
        std::fs::write(&input_path, "ψ 10 γίγνεται.").unwrap();

        let result = run_tests(&input_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // The underlying error bubbles up.
        assert!(err_msg.contains("Semantic error") || err_msg.contains("Σφάλμα"));
    }

    #[test]
    fn test_print_test_results_coverage() {
        use std::os::unix::process::ExitStatusExt;

        let success_output = std::process::Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };
        let fail_output = std::process::Output {
            status: std::process::ExitStatus::from_raw(256),
            stdout: b"failures:\n\n---- test stdout ----\nerr\n\nfailures:\n    test\n".to_vec(),
            stderr: b"stderr output".to_vec(),
        };

        let empty_results = vec![];
        let some_results = vec![
            TestResult { name: "test_ok".to_string(), status: TestStatus::Ok },
            TestResult { name: "test_failed".to_string(), status: TestStatus::Failed },
            TestResult { name: "tests::test_ignored".to_string(), status: TestStatus::Ignored },
        ];

        // We capture output implicitly by just running these, since they write to stdout.
        // It's mostly to trigger the code paths for coverage.
        print_summary_table(true, true);
        print_summary_table(false, true);

        print_results_table(&empty_results);
        print_results_table(&some_results);

        print_failure_details("", &success_output);
        print_failure_details("failures:\n\n---- t stdout ----\nmsg\n\nfailures:\n    t\n", &fail_output);
        print_failure_details("raw failure", &fail_output);

        print_test_results(&some_results, &fail_output, "failures:\n\n---- t stdout ----\nmsg\n\nfailures:\n    t\n");
    }
}
>>>>>>> REPLACE
```
2. **Verify test added**:
   - Use `read_file` to verify the modified contents of `src/tools/tester.rs`.
3. **Run Clippy & Fmt**:
   - Use `run_in_bash_session` to execute `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt --all`.
4. **Execute tests**:
   - Use `run_in_bash_session` to execute `cargo test` across the workspace to verify everything works.
5. **Pre-commit**:
   - Complete pre commit steps to ensure proper testing, verification, review, and reflection are done.
6. **Submit PR**:
   - Use the `submit` tool to update the PR.
