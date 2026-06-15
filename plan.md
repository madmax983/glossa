1. **Prepare test showcasing the stack overflow**
   - We have already created the integration test `tests/havoc_semantic_stack_overflow.rs` containing a test `havoc_semantic_clone_drop_stack_overflow` which builds a very deep AST tree and then safely forces a stack overflow upon clone/drop. This organically crashes the test runner, which is what the Havoc persona mandates.
2. **Review the Pre-commit steps**
   - Run `pre_commit_instructions` tool.
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
3. **Submit the PR**
   - Submit the PR detailing the crash, with title "👺 Havoc: Stack Overflow in AnalyzedExpr Clone and Drop", description including "🧨 **The Trigger:** Deeply nested AST bypasses stack limit", "📉 **The Stack Trace:** (stack overflow)", "🔬 **Reproduction:** Run `cargo test --test havoc_semantic_stack_overflow`", and "😈 **Comment:** Warden missed the semantic model. Enjoy the crash."
