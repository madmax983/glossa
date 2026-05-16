## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `GnomonVisitor` and `AuditorVisitor` used object-oriented patterns with state structures combining traversal methods and simple tracking state.
**Cut:** Flattened the state objects into pure procedural recursive functions passing mutable references to `max_depth`, `current_depth`, and various `hashmap` metrics.
**Saved:** Replaced verbose trait-like object-oriented abstractions with flat, idiomatic procedural Rust functions, drastically reducing code boilerplate.
