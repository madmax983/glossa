1. **Verify the codebase**
   - Use `run_in_bash_session` to run `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo fmt --all`.

2. **Document the reduction**
   - Use `run_in_bash_session` to append a Razor log entry to `.jules/razor.md`:
   ```bash
   cat << 'EOF' >> .jules/razor.md

   ## [Reduction]
   **Bloat:** `GnomonVisitor` and `AuditorVisitor` in `src/tools` used object-oriented builder patterns for simple AST traversal logic.
   **Cut:** Flattened the objects into pure procedural functions passing mutable references for state (like `usage_count`, `mutation_count`, etc.).
   **Saved:** Replaced localized object-oriented abstractions with standard procedural Rust functions.
   EOF
   ```

3. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**

4. **Submit PR**
   - Submit the PR with the following tool call arguments:
   `branch_name`: "razor-flatten-visitors"
   `title`: "🔨 Razor: Flattened GnomonVisitor and AuditorVisitor"
   `commit_message`: "Refactored GnomonVisitor and AuditorVisitor into pure procedural functions"
   `description`: "The `GnomonVisitor` and `AuditorVisitor` structs were localized object-oriented builder patterns used purely for single-pass AST traversal. Following Razor's essentialist philosophy, these have been flattened into pure procedural functions (`visit_statement` and `visit_expr`), passing their respective state variables (like `max_depth`, `usage_count`, etc.) as explicit mutable references. This eliminates unnecessary abstractions and conforms to a simpler, more procedural style."
