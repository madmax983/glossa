## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `GnomonVisitor` in `src/tools/gnomon.rs` and `AuditorVisitor` in `src/tools/auditor.rs` used object-oriented builder patterns for AST traversal.
**Cut:** Flattened both objects into pure procedural functions `visit_statement` and `visit_expr`, passing mutable references to tracking state (like `current_depth`, `max_depth`, or HashMaps).
**Saved:** Eliminated two single-use Visitor structs, drastically simplifying the code into straightforward recursive functions, adhering to procedural simplicity.
