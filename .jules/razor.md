## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** Thin wrapper functions in `Scope` (`lookup_variable`, `is_function`, etc.).
**Cut:** Inlined `lookup_variable` directly into `lookup_binding`.
**Saved:** Reduced unnecessary indirection in scope lookups.

## [Reduction]
**Bloat:** Fragmented small files (`src/limits.rs`).
**Cut:** Merged the 47-line `limits.rs` file directly into `src/ast.rs` where the parser limits conceptually belong.
**Saved:** Removed an entire module from the compiler structure, keeping related constraints flat.

## [Reduction]
**Bloat:** Single-use helper function (`get_first_word`).
**Cut:** Inlined the logic directly into its only caller `analyze_control_flow`.
**Saved:** Flattened control flow parsing logic into a single cohesive block.
