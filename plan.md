1. **Explore `src/codegen.rs` to fix `.unwrap()`, `.expect()`, and panic issues.**
   - Wrote three tests (`tests/sentry_codegen_bounds.rs`, `tests/sentry_codegen_arithmetic.rs`, `tests/sentry_codegen_unwrap.rs`) to verify that the compiler injects `.expect()` and `panic!` calls into the generated code properly to handle runtime errors gracefully.

2. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
3. **Submit the change.**
   - I will submit the change with a descriptive commit message "🛡️ Sentry: [test coverage improvement] for codegen panic checks".
