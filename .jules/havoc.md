**[AnalyzedExpr Drop Stack Overflow]
**The Trigger:** A deeply recursive `AnalyzedExpr` tree overflowing the stack during Drop execution or Clone logic.
**The Stack Trace:** Runtime crash (`thread '<unknown>' has overflowed its stack / signal: 6, SIGABRT: process abort signal`).
**Reproduction:** Run `cargo test --test havoc_clone_drop_model` which builds a tree of depth 50,000 and explicitly calls `clone` and `drop`.
**Comment:** We assumed the parser depth limits (`check_recursion_depth`) were sufficient to stop AST nesting, but `AnalyzedExpr` can bypass this check and stacker logic on AST `Drop` didn't carry over into the semantic analysis phase. By implementing `Drop` manually on `AnalyzedExpr` utilizing `stacker::maybe_grow`, we can safely collapse recursive enum payloads into dummy variants without throwing `E0509` (Cannot move out of type which implements Drop) that would occur if we put `Drop` on `AnalyzedExprKind` directly.
