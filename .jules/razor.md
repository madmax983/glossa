## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** `GnomonVisitor` and `AuditorVisitor` in `src/tools/gnomon.rs` and `src/tools/auditor.rs` used object-oriented visitor patterns with struct wrappers for AST traversal.
**Cut:** Flattened the visitors into pure procedural functions `visit_statement` and `visit_expr` that directly pass mutable state parameters (`current_depth`, `max_depth` for Gnomon; `usage_count`, `mutation_count`, `mutable_vars` for Auditor).
**Saved:** Eliminated unnecessary structs and `impl` blocks, simplifying the traversal logic into straightforward procedural calls.
