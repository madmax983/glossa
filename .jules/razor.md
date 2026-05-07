## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Fixes]
**Bloat:** None, bug fix.
**Cut:** Fixed fallback evaluations in semantic conversion for unknown variables and implemented missing verb checking.
**Saved:** Avoids silent errors and ICEs.
