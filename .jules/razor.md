## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** Object-oriented Visitor pattern structs (`AuditorVisitor`, `GnomonVisitor`) that existed solely to hold transient state during AST traversal.
**Cut:** Flattened the traversal logic into purely procedural recursive functions that pass state explicitly (maps) or return it directly (depth), eliminating the single-use structs entirely.
**Saved:** ~50 lines of boilerplate code and cognitive load from unnecessary struct implementations.
