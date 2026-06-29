📖 Chapter: `src/codegen.rs` documentation structure
💡 Insight: The `to_rust_type` function was missing documentation according to `rustdoc` because a `use std::fmt::Write;` statement was placed between its doc comment and the function signature. This detaches the doc comment, so I moved the import to the top of the file to fix the warning.
🧪 Example: N/A - structural fix for existing docs.
🖼️ Preview: N/A
