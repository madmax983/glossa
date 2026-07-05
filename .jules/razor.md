## [Reduction]
**Bloat:** Standard implicit recursive drop causing stack overflow panics when traversing deep compiler ASTs.
**Cut:** Flattened the drop structure using `std::mem::swap` combined with `stacker::maybe_grow` inside a manual `impl Drop` for `AnalyzedExpr` and `AnalyzedStatement`, breaking deep recursion without requiring complex pointer types.
**Saved:** Hundreds of kilobytes of call stack space and fully mitigated deep recursive AST panic vulnerabilities while minimizing object-oriented memory wrappers.
