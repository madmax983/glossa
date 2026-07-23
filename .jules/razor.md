## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** `GnomonVisitor` in `src/tools/gnomon.rs` and `AuditorVisitor` in `src/tools/auditor.rs` used struct-based visitors and state tracking for simple traversals. `TraitMethodParts` in `src/codegen.rs` was a single-use struct for grouping function return values.
**Cut:** Flattened the AST visitors into pure procedural recursive functions passing explicit state. Replaced `TraitMethodParts` with a standard Rust tuple return type.
**Saved:** Removed unnecessary struct definitions and boilerplate object-oriented patterns in favor of procedural simplicity.
