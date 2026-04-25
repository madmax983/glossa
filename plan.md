1. **Identify Missing Coverage:**
   - The `codecov/patch` CI check failed because it remains at 85.71% coverage, meaning the test I added didn't hit the uncovered lines inside `parse_return_expression`.
   - The uncovered lines are likely the `?` errors on `feed_expr_to_assembler_with_context` or `finalize()`.
   - If I used `Expr::Word(Word::new("καί"))`, it doesn't fail during `feed_expr` or `finalize`! Conjunctions are valid in assembly, they just don't extract well, but maybe the assembler doesn't error out on them?
   - Wait, let me check the exact coverage report for `src/semantic/control_flow.rs`.

2. **Run `cargo llvm-cov` on `control_flow.rs`:**
   - I need to see EXACTLY which lines are missed. I will run `cargo llvm-cov --show-missing-lines` or generate an HTML report to see which lines in `control_flow.rs` are red.

3. **Write Targeted Test:**
   - Write a test that explicitly triggers the exact error condition missed. E.g. `asm.finalize()?` erroring out if there is no verb, or if the expression recursively exceeds max depth.

4. **Verify & Submit:**
   - Fix coverage, run llvm-cov, submit.
