## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** Unused variable `input` warning in `src/main.rs` due to the `Commands::Gnomon` match arm missing consumption of destructured variable inside `#[cfg(not(feature = "nova"))]` block.
**Cut:** Added `let _ = input;` inside the non-nova block to suppress unused variable warning without suppressing all such warnings.
**Saved:** Suppressed a compiler warning gracefully, maintaining strict zero-warning policy without hiding legitimate issues.
