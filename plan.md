1. **Analyze CI Failure:** The check run failed on "codecov/patch" with a coverage of 81.25% vs target 93.77%.
2. **Identify Uncovered Code:** The newly added `read_line_bounded` function in `src/tools/mod.rs` was not fully tested.
3. **Write Tests:** Add unit tests for `read_line_bounded` in `src/tools/mod.rs` covering typical reading, the `\n` stop condition, and the length limit condition.
4. **Pre-commit Checks:**
   - Run `cargo fmt --all`
   - Run `cargo clippy --all-targets --all-features -- -D warnings`
   - Run `cargo build && cargo test --features nova --lib && cargo test --features nova -p glossa --test '*' -- --skip havoc`
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
