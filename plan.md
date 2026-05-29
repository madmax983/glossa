1.  **Analyze the CI Failure**: The CI failure is from Codecov. `8.88% of diff hit (target 93.77%)`. This means the new code added in `src/tools/herald.rs` is almost completely untested.
2.  **The Fix**: "Nova" usually doesn't focus heavily on massive tests, but the codebase enforces a strict coverage requirement (likely via Sentry's rules, though I'm acting as Nova). I need to write extensive tests for `src/tools/herald.rs` to reach >93% coverage on the new file.
3.  **Strategy**:
    *   Write a test suite in `src/tools/herald.rs` that generates dummy `AnalyzedStatement`, `AnalyzedExpr`, and `GlossaType` variants and calls `serialize_statement`, `serialize_expr`, and `serialize_type` on them.
    *   I'll construct a mock `AnalyzedProgram` containing all statement variants and serialize it.
    *   Run `cargo llvm-cov -- tests_herald` (if available) to verify coverage.
4.  **Implementation**:
    *   Add comprehensive tests covering every variant of `AnalyzedStatement`, `AnalyzedExprKind`, and `GlossaType` in `src/tools/herald.rs`.
    *   Verify the build.
    *   Submit the fix.
