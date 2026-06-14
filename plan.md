1. **Apply custom `Clone` and `Drop` to semantic AST.** Replace `#[derive(Clone)]` on `AnalyzedExpr`, `AnalyzedExprKind`, `AnalyzedStatement`, and `AnalyzedMethod` with custom implementations that use `stacker::maybe_grow` to prevent stack overflow.
2. **Update `sentry_conversion_tests.rs`.** Refactor tests to match by reference (`ref`) instead of moving fields out, since moving out of a type that implements `Drop` is illegal in Rust.
3. **Update `havoc_semantic_stack_overflow.rs`.** Change the havoc test to expect success, since the stack overflow vulnerability is now successfully mitigated.
4. **Update `.jules/warden.md`.** Add a journal entry detailing the threat and defense.
5. **Pre-commit tasks.** Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
6. **Submit PR.** Submit PR using the Warden persona format.
