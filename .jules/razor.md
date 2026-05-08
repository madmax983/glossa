## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `TestCase` struct in test modules.
**Cut:** Simplified the one-time `struct TestCase { ... }` in table-driven tests into standard tuples instead of one-off custom structures to minimize boilerplate.
**Saved:** Removed extraneous struct definitions and property accessors in multiple files `src/morphology/declension.rs` and `src/tools/tester.rs`.
