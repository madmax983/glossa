## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `GnomonVisitor` in `src/tools/gnomon.rs` and `AuditorVisitor` in `src/tools/auditor.rs` used object-oriented builder patterns for simple AST traversals.
**Cut:** Flattened both objects into pure procedural functions `calculate_max_depth`, `audit_statement`, and `audit_expr`, passing only necessary state via mutable references to eliminate dead state parameters in recursive calls.
**Saved:** Replaced verbose, localized object-oriented abstractions with standard procedural Rust functions, satisfying clippy lints and reducing lines of code.
