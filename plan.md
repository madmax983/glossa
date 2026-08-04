1. **Refactor `print_test_results` in `src/tools/tester.rs`**:
    - `print_test_results` is a "God Function" (over 100 lines, currently 111 lines, starting from line 283).
    - It handles formatting success/failure summary tables, formatting individual test result tables, and extracting/formatting detailed error messages.
    - We will extract this into smaller helper functions:
        - `print_summary_table(success: bool)`
        - `print_results_table(results: &[TestResult])`
        - `print_failure_details(stdout: &str, test_output: &std::process::Output)`
    - We will keep the original logic and ensure there are no behavior changes, just separating concerns for readability.
2. **Execute tests**:
    - Run `cargo test` to verify everything works and no functionality is broken.
3. **Run Clippy & Fmt**:
    - Run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt --all`.
4. **Pre-commit**:
    - Complete pre-commit steps to ensure proper testing, verifications, reviews and reflections are done.
5. **Submit PR**:
    - Use the submit tool to create a PR: "⚒️ Forge: [refactor name]".
