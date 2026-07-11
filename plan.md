1. Use `run_in_bash_session` to execute `cat src/tools/gnomon.rs src/tools/auditor.rs` to verify the flattened structures.
2. Use `run_in_bash_session` to append a journal entry to `.jules/razor.md` using `cat << 'EOF' >> .jules/razor.md` containing the reduction block:
   ```markdown
   ## [Reduction]
   **Bloat:** `GnomonVisitor` and `AuditorVisitor` used object-oriented builder patterns for simple traversals.
   **Cut:** Flattened the objects into pure procedural functions passing mutable references to state.
   **Saved:** Replaced localized object-oriented abstractions with standard procedural Rust functions.
   ```
3. Use `run_in_bash_session` to execute `tail -n 10 .jules/razor.md` to confirm the journal entry was appended correctly.
4. Use `run_in_bash_session` to execute `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo fmt --all`.
5. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
6. Use `submit` to create the PR:
   Title: `🪒 Razor: Flatten Visitor structs into procedural functions`
   Description:
   ```markdown
   ## [Reduction]
   **Bloat:** `GnomonVisitor` and `AuditorVisitor` used object-oriented builder patterns for simple traversals.
   **Cut:** Flattened the objects into pure procedural functions passing mutable references to state.
   **Saved:** Replaced localized object-oriented abstractions with standard procedural Rust functions.
   ```
