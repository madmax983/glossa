1. Verify Code
   - Use `run_in_bash_session` to execute `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings` to ensure the changes are correct and have not introduced regressions.
2. Pre-commit Steps
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
3. Submit PR
   - Submit the PR using `submit` with the exact branch name "forge-refactor", title "⚒️ Forge: Refactoring feature gates, contexts, and iterators", and description:
```markdown
🚮 Smell:
- `src/main.rs` contains highly repetitive `#[cfg(feature = "nova")]` boilerplate blocks in `match cli.command` which bloated the function size.
- `src/morphology/disambiguation.rs` has deeply vertical and repetitive struct initialization in match arms for `analyze_article`.
- `src/semantic/resolver.rs` relies on manual `for` loop iterations with early returns for map lookups rather than idiomatic functional pipelines.

✨ Solution:
- Introduced a `run_experimental!` macro in `src/main.rs` to encapsulate the conditional feature logic, drastically flattening the `match` block.
- Refactored `analyze_article` in `src/morphology/disambiguation.rs` to flatten struct literals into a single-line form.
- Refactored lookup methods in `src/semantic/resolver.rs` to utilize `.iter().rev().find_map(...)`.

🧼 Benefit:
- Improves code density and readability by stripping out repetitive boilerplate.
- Converts manual loops into robust, chained iterator combinators.

🛡️ Verification: Tests passed. No logic changed.
```
