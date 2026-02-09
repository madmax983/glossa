## [Reduction]
**Bloat:** `struct Clause { expressions: Vec<Expr> }` wrapper around a vector.
**Cut:** Removed `Clause` struct and flattened `Statement::Regular` to use `Vec<Vec<Expr>>`.
**Saved:** 1 struct definition, removed a layer of indirection in AST traversal.
