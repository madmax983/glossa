1. **Refactor `src/parser/recursion.rs`:**
   - I will modify `src/parser/recursion.rs` using `run_in_bash_session` to rewrite `check_recursion_depth`.
   - The current `check_recursion_depth` function has deeply nested `if/else` and `match` blocks. It currently manually iterates `i` over byte length.
   - I will extract logic into private helper functions (like `check_keywords` or similar) or simplify the loop with guard clauses, flattening the logic to improve readability. I will ensure no runtime behavior changes (especially the iteration over bytes with varying jump lengths).
2. **Verify changes:**
   - Run `cargo fmt --all`.
   - Run `cargo clippy --all-targets --all-features -- -D warnings`.
   - Run `cargo test` to ensure tests pass.
3. **Log learning:**
   - Log the refactoring insights to `.jules/forge.md` via `echo "..." >> .jules/forge.md`.
4. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
5. **Submit Pull Request:**
   - Use `request_code_review` or `submit` with the exact PR Title and Description.
     - Title: `⚒️ Forge: Flatten check_recursion_depth`
     - Description with `🚬 Smell:`, `✨ Solution:`, `🧱 Benefit:`, `🛡️ Verification:`.
