## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** Unnecessary localized `TestCase` struct definitions for table-driven tests in `src/tools/tester.rs`, `src/morphology/declension.rs`, and `tests/sentry_participle_tests.rs`.
**Cut:** Flattened the `TestCase` struct definitions and replaced them with simple tuple vectors that are natively destructured in test loops.
**Saved:** Removed extraneous localized types and explicit struct initialization boilerplates across multiple files, increasing code readability.
