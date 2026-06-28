## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** Object-oriented `Visitor` pattern with `GnomonVisitor` and `AuditorVisitor` structs that maintain state across traversal but act effectively as procedural functions with heavy coupling.
**Cut:** Flattened `GnomonVisitor` into procedural `calculate_depth` function, and `AuditorVisitor` into `AuditorState` data struct + independent procedural `visit_x` functions.
**Saved:** ~30 lines of code, significantly reduced cognitive load by removing single-use object-oriented structs and explicit deep `self` state.
