## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `AuditorVisitor` in `src/tools/auditor.rs` and `GnomonVisitor` in `src/tools/gnomon.rs` used object-oriented visitor structs for simple state tracking.
**Cut:** Flattened both visitors into pure procedural functions `visit_statement` and `visit_expr` passing explicit mutable references for state.
**Saved:** Replaced localized object-oriented abstractions with standard procedural Rust functions.
