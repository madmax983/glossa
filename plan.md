1.  **Analyze the CI Failure**: The CI failures indicate `error[E0308]: mismatched types` in `src/tools/herald.rs`. Specifically, the tests use `AnalyzedStatement::For`, `Match`, and `Return` without properly boxing the `AnalyzedExpr` inner values. E.g., `let stmt = AnalyzedStatement::Return { value: Some(dummy_expr.clone()) };` needs `value: Some(Box::new(dummy_expr.clone()))`. Also `TestDeclaration` has `name` expected as `String` but `SmolStr` was provided.
2.  **The Fix**: I need to update the mock test definitions in `src/tools/herald.rs` around line 664 to correctly use `Box::new()` for AST node composition where the enums expect boxed values, and use `.to_string()` for `name` in `TestDeclaration`.
3.  **Strategy**:
    *   Find the exact failing tests block in `src/tools/herald.rs`.
    *   Fix `AnalyzedStatement::For` iterator from `dummy_expr` to `Box::new(dummy_expr)`.
    *   Fix `AnalyzedStatement::Match` scrutinee from `dummy_expr` to `Box::new(dummy_expr)`.
    *   Fix `AnalyzedStatement::Return` value from `Some(dummy_expr)` to `Some(Box::new(dummy_expr))`.
    *   Fix `AnalyzedStatement::TestDeclaration` name from `SmolStr::new("t")` to `"t".to_string()`.
    *   Run `cargo test` and `cargo check` locally.
    *   Submit the fix.
