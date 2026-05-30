## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** Unused variables warning in `src/main.rs` due to the `input` field in `Commands::Gnomon` being ignored when the `nova` feature is disabled.
**Cut:** Prefixed the `input` variable with an underscore (`_input`) in the match arm to explicitly mark it as unused and suppress the warning without hiding other potential issues.
**Saved:** Suppressed 1 compiler warning.
