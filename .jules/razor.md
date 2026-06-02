## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `unused_variables` warning for `input` in `Commands::Gnomon` match arm when compiled without `--all-features`.
**Cut:** Renamed `input` to `_input` in the match arm pattern to silence the warning without using `#[allow(unused_variables)]`.
**Saved:** Suppressed a compiler warning gracefully, maintaining `#[deny(warnings)]` compliance and codebase cleanliness.
