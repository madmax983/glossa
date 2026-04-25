1. **Identify Missing Coverage:**
   - The CI failed with `codecov/patch` because the newly added code in `src/semantic/control_flow.rs` doesn't have sufficient test coverage. 87.50% is below the required 93.95% threshold.
   - I need to check the code coverage of `src/semantic/control_flow.rs` using `cargo llvm-cov` to identify which lines of the new `parse_return_expression` logic are not being executed by tests.
   - Wait, `codecov/patch` means lines changed in my patch. I added:
     ```rust
     let mut asm = crate::semantic::assembly::Assembler::new();
     let mut ctx = crate::morphology::DisambiguationContext::new();
     for term in words {
         crate::semantic::expressions::feed_expr_to_assembler_with_context(&mut asm, term, &mut ctx)?;
     }
     let asm_stmt = asm.finalize()?;
     match crate::semantic::conversion::extract_value(&asm_stmt, scope) {
         Ok((expr, _)) => Ok(expr),
         Err(e) => Err(e),
     }
     ```
   - What could be failing coverage?
     - The `Err(e)` branch of `match extract_value`.
     - The `?` on `finalize()`.
     - The `?` on `feed_expr_to_assembler_with_context`.
   - I will add a new unit test in `src/semantic/control_flow.rs` or `tests/` that feeds an invalid expression to the return statement, causing `finalize()` or `extract_value()` to return an `Err`.

2. **Add Tests:**
   - Add a test that causes `feed_expr_to_assembler_with_context` to fail (e.g. an invalid expression node, though this is hard to trigger directly).
   - Add a test that causes `extract_value` to fail (e.g., returning multiple expressions or something invalid). Wait, returning a verb that is not a value? `δός λέγε.` (Return say). Or an incomplete expression `δός καὶ.` (Return and).
   - This will hit the `Err(e)` branch.

3. **Verify Coverage:**
   - Run `cargo llvm-cov` to ensure the patch coverage reaches 100%.

4. **Submit:**
   - Pre-commit verification.
   - Submit the updated code.
