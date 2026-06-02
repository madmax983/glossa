1. **Refactor `print_test_results` in `src/tools/tester.rs`**
    - Use `run_in_bash_session` with exact `sed` command to replace the `print_test_results` function in `src/tools/tester.rs` (lines 283-393) with the explicit implementation of the new helper functions: `print_test_results`, `print_header`, `print_summary`, `print_results_table`, and `print_failures`.
2. **Verify refactoring**
    - Use `run_in_bash_session` to run `cargo check` to confirm the file was modified properly and is free of syntax errors.
3. **Run the test suite**
    - Run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --lib`, and `cargo test --test '*' -- --skip havoc`.
4. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
5. **Submit the Pull Request**
    - Use the `request_code_review` tool to create a PR.
    - Title `⚒️ Forge: Extract print_test_results helpers`.
    - Description:
        * 🚮 Smell: The `print_test_results` function is over 110 lines long and handles multiple responsibilities.
        * ✨ Solution: Extracted the logic into smaller, named helper functions.
        * 🧼 Benefit: Reduces cognitive load and improves readability.
        * 🛡️ Verification: Tests passed. No logic changed.
