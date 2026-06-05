## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `GnomonVisitor` in `src/tools/gnomon.rs` used an object-oriented visitor pattern to track maximum loop depth.
**Cut:** Flattened the object into a pure recursive function `calculate_loop_depth(statements: &[AnalyzedStatement], current_depth: usize) -> usize` without mutable state passing.
**Saved:** Removed unnecessary struct declaration, eliminated stateful traversal, and simplified test setups.
