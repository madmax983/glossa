1. Use `run_in_bash_session` with a bash heredoc to write the complete, finalized Rust code containing all necessary logic and tests for `tests/havoc_return_silence.rs` to expose the silent zero bug in `parse_return_expression`.
2. Verify that the new test file was written correctly using `cat tests/havoc_return_silence.rs` in `run_in_bash_session`.
3. Use `run_in_bash_session` to run `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo fmt --all` to verify the codebase and see the test fail.
4. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
5. Submit the change with a PR title `👺 Havoc: The Silent Zero Bug Exposed` and description formatting with `🧨 **The Trigger:**`, `📉 **The Stack Trace:**`, `🧪 **Reproduction:**`, and `😈 **Comment:**`.
