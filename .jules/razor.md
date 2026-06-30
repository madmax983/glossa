## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `GnomonVisitor` and `AuditorVisitor` used unnecessary struct-based object-oriented Visitor patterns to maintain basic traversal state.
**Cut:** Replaced the structs and `impl` blocks with standard pure procedural functions (`visit_statement_for_depth`, `visit_statement_for_audit`, `visit_expr_for_audit`) that pass state references explicitly.
**Saved:** Removed local OOP boilerplate in favor of clear, flat recursive functions.
