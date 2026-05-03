## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** Duplicated `TestCase` structs inside multiple test functions in `src/tools/tester.rs`.
**Cut:** Extracted the struct definition to the top of the `tests` module.
**Saved:** Reduced boilerplate and enforced DRY principles for test configurations.
