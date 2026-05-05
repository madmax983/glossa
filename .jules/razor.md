## [Reduction]
**Bloat:** State-tracking visitor struct (`GnomonVisitor`) for simple recursion.
**Cut:** Flattened into a pure recursive function `calculate_max_depth`.
**Saved:** ~50 lines of boilerplate and mutable state cognitive load.

## [Reduction]
**Bloat:** Layer lasagna in error definitions (`src/errors/assembly.rs`).
**Cut:** Merged `assembly.rs` directly into `src/errors/mod.rs`.
**Saved:** Unnecessary file nesting and module exports.

## [Reduction]
**Bloat:** Single-use struct `TraitMethodParts` in `src/codegen.rs` just to return two strings.
**Cut:** Replaced with a standard Rust tuple `(String, Option<String>)`.
**Saved:** Unnecessary struct definition and boilerplate.
