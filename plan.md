1. We need to add tests for the generated `.expect(...)` panics from `generate_checked_op` when used with BinaryOps (e.g., Add, Sub, Mul, Div, Mod).
2. The `tests/sentry_codegen_perf.rs` currently tests:
   - `test_codegen_runtime_unwrap_panic` (for `.expect("attempted to unwrap an empty value")`)
   - `test_codegen_runtime_large_index_panic` (for `.expect("index out of bounds: too large")`)
   - `test_codegen_runtime_index_panic` (for unchecked `[idx]`)
   - `test_codegen_runtime_neg_panic` (for `.expect("arithmetic overflow")` in `generate_unary_op`)
   - `test_codegen_runtime_negative_index_panic` (for `panic!("index out of bounds: negative index {}")`)
3. However, there are no runtime integration tests that cover `generate_checked_op` specifically triggered by binary operations.
4. Add `test_codegen_runtime_add_overflow_panic` to `tests/sentry_codegen_perf.rs` to compile a program with `i64::MAX + 1` (or equivalent via `AnalyzedExpr`) and assert it panics with "arithmetic overflow" at runtime.
5. Add `test_codegen_runtime_div_by_zero_panic` to `tests/sentry_codegen_perf.rs` to compile a program with `1 / 0` and assert it panics with "arithmetic overflow" or similar (Wait, div by zero produces "division by zero" via `checked_div`).
6. I will add `test_codegen_runtime_add_overflow_panic` and `test_codegen_runtime_div_by_zero_panic` to `tests/sentry_codegen_perf.rs`.
7. Complete pre commit steps to ensure proper testing, verification, review, and reflection are done.
8. Submit the change.
