## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `GnomonVisitor` and `AuditorVisitor` used the Visitor pattern with `struct` implementation to do basic recursive AST transversals.
**Cut:** Flattened these Visitor structs into pure procedural recursive functions, directly mutating state passed in as variables (`current_depth`, `max_depth` for Gnomon, and HashMaps for Auditor).
**Saved:** Removed unnecessary struct instantiations and boilerplates, reducing the object-oriented approach to simple procedural iterations.
