## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** `TestCase` struct defined for each table test in `src/morphology/declension.rs` and `src/tools/tester.rs`
**Cut:** Flattened test table structure by removing the local `TestCase` structs and using simple tuples.
**Saved:** Localized one-off abstractions replaced with native tuples, saving multiple lines of code and removing redundant struct definitions.
