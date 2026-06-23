1.  **Refactor `tell_type` in `src/tools/narrator.rs`**:
    - Right now, `tell_type` builds up type strings using recursive `format!` calls, causing unnecessary heap allocations for nested types like `Map<Option<Number>, String>`.
    - Following the pattern from `.jules/bolt.md` for "Optimizing recursive type formatting", we will change `tell_type` to use a `write!` approach with a single pre-allocated `String` buffer.
    - We will define `write_tell_type(ty: &GlossaType, out: &mut String) -> std::fmt::Result` to handle the recursive logic.
    - `tell_type` will simply allocate a `String::with_capacity(32)` and call `write_tell_type(&ty, &mut buf)`.
    - `format_types` will be updated to write directly into the buffer, completely eliminating intermediate strings!

2.  **Verify the change**:
    - Run `cargo fmt` and `cargo clippy`.
    - Run `cargo test` to ensure we didn't break any narrator functionality or tests.

3.  **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done**.
    - Invoke `pre_commit_instructions` and follow the provided steps.

4.  **Submit the PR**:
    - Title: `⚡ Bolt: [Zero-Cost Abstract type string builder]`
    - Description: Following the Bolt format, specifying the removal of intermediate `format!` allocations via `std::fmt::Write`.
