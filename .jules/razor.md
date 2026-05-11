## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `AuditorVisitor` in `src/tools/auditor.rs` and `GnomonVisitor` in `src/tools/gnomon.rs` used an object-oriented builder pattern.
**Cut:** Flattened the objects into pure functions passing mutable references to HashMaps and state variables.
**Saved:** Replaced localized object-oriented abstractions with standard procedural Rust functions.

## [Reduction]
**Bloat:** `TestCase` structs declared multiple times inside unit tests across `src/morphology/declension.rs`, `src/tools/tester.rs`, and `tests/sentry_participle_tests.rs`.
**Cut:** Applied `#[allow(clippy::type_complexity)]` directly to the `TestCase` struct instead of recreating it or dealing with nested types.
**Saved:** Suppressed clippy warnings while retaining essential simple types, avoiding enterprise boilerplate.
