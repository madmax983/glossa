## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `GnomonVisitor` and `AuditorVisitor` structs were unnecessarily object-oriented for pure structural AST traversals.
**Cut:** Flattened both visitors into pure procedural functions (e.g. `calculate_depth`, `visit_statement`, `visit_expr`) that pass mutable state explicitly.
**Saved:** Removed two locally defined `Visitor` abstractions, improving consistency with procedural Rust practices and removing unnecessary boilerplate classes for traversal.
