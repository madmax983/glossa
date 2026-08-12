## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `unused_variable` warning in `Commands::Gnomon` branch due to lack of `let _ = input;` when the `nova` feature is not enabled.
**Cut:** Added `let _ = input;` to the `#[cfg(not(feature = "nova"))]` conditional branch, similarly to other commands (like `Scholar` or `Haruspex`).
**Saved:** Suppressed a localized compiler warning with an explicit empty assignment.
