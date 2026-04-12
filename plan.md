1. **Remove `#[derive(Clone)]` from `Statement` in `src/ast.rs` and implement custom `Clone` with `stacker::maybe_grow`**
   - The memory context states: "To prevent stack overflow panics on deeply nested compiler nodes in Glossa (like `AnalyzedExpr`, `Expr`, or `GlossaType`), always implement custom `Drop`, `Clone`, and `Debug` (via `std::fmt::Debug`) traits using `stacker::maybe_grow` or iterative routines instead of relying on the default recursive compiler implementations."
   - The same applies to `AnalyzedStatement`, `AnalyzedExpr`, `AnalyzedExprKind`, and `GlossaType` in `src/semantic/model.rs` and `src/semantic/types.rs` which I already started modifying (though currently it failed compiling because I didn't add it back properly).
   - I will revert my broken `src/semantic/model.rs` and `src/semantic/types.rs` changes to a clean state.
   - I will implement `Clone` and `Drop` manually using `stacker::maybe_grow` for `Statement`, `AnalyzedStatement`, `AnalyzedExpr`, `AnalyzedExprKind`, and `GlossaType`.

2. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done**
   - Run `pre_commit_instructions`.

3. **Submit the change**
   - Commit using a descriptive message and submit.
