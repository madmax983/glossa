## [Reduction]
**Bloat:** `generate_expr_*` wrapper functions in `src/codegen.rs` that wrapped simple `quote!` macro calls.
**Cut:** Inlined the functions directly into the `generate_expr` match statement.
**Saved:** Removed 8 unnecessary functions, simplifying the file by reducing 40 lines of abstraction boilerplate.
