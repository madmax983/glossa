1. **Analyze CI Failure:** The check run failed on "Format Check" running `cargo fmt --all -- --check`. The diff shows missing trailing commas in the array initializing the `Table` rows.
2. **Fix `src/tools/tester.rs`:** Run `cargo fmt --all` to automatically apply the formatting changes required to fix the trailing comma issues.
3. **Verify:** Ensure `cargo fmt --all -- --check` passes.
4. **Submit PR.**
