## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** `GnomonVisitor` in `src/tools/gnomon.rs` used an object-oriented visitor struct for tracking a simple depth value.
**Cut:** Flattened the object into a pure procedural function `visit_statement` passing `current_depth` and `max_depth`.
**Saved:** Replaced localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `AuditorVisitor` in `src/tools/auditor.rs` used an object-oriented visitor struct with multiple HashMaps and HashSets for tracking state during analysis.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `usage_count`, `mutation_count`, and `mutable_vars` state.
**Saved:** Replaced localized object-oriented abstraction with standard procedural Rust functions.
