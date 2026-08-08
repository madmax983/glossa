1. **Refactor `main` function in `src/main.rs`**
   - Use the `replace_with_git_merge_diff` tool to extract the large `match cli.command` block in `src/main.rs:main` into a separate helper function `run_command` in `src/main.rs`.
   - The `run_command` function will take `cli.command` as an argument and execute the corresponding command.
   - This will significantly reduce the size of the `main` function and improve readability, addressing the "God Function" smell.
2. **Verify Changes**
   - Use `run_in_bash_session` to verify the new `main` function using `cat src/main.rs`.
3. **Run Checks**
   - Use `run_in_bash_session` to run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`.
4. **Complete pre-commit steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
5. **Submit PR**
   - Use `submit` tool to create the PR. The description will follow the ⚒️ Forge format.
