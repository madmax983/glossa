## [Reduction]
**Bloat:** Manual AST traversal (`AnalyzedStatement` and `AnalyzedExpr`) duplicated across multiple visitor-like structs (`GnomonVisitor`, `AuditorVisitor`, `ProgramStats`).
**Cut:** Created a standard procedural `Visitor` trait in `src/semantic/visitor.rs` with default recursive `walk_` implementations. Refactored `GnomonVisitor`, `AuditorVisitor`, and `ProgramStats` to implement this trait, overriding only the necessary methods.
**Saved:** Hundreds of lines of duplicate `match` statements over AST nodes, simplifying logic and enforcing consistency.
