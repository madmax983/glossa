## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `GnomonVisitor` and `AuditorVisitor` in `src/tools` used object-oriented builder patterns for simple AST traversal logic.
**Cut:** Flattened the objects into pure procedural functions passing mutable references for state (like `usage_count`, `mutation_count`, etc.).
**Saved:** Replaced localized object-oriented abstractions with standard procedural Rust functions.
