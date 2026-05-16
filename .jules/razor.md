## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** Conditionally compiled commands in `src/main.rs` used variable binding and `{ let _ = input; ... }` to silence unused variable warnings, making code hard to read.
**Cut:** Conditionally compiled the entire match arms, using structural wildcard `{ .. }` for negative feature fallbacks to implicitly ignore inputs.
**Saved:** Reduced syntactic noise and boilerplate code handling feature flags.
