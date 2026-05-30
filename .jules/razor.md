## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** Unnecessary `FxHashMap` and `FxHashSet` optimization
**Cut:** Replaced with standard `HashMap` and `HashSet` from `std::collections`. The `rustc_hash` crate provides a performance improvement when cryptographic security (DoS prevention) is unnecessary, however, replacing the standard map with it throughout tools creates excessive complexity unless there is a verified hot path.
**Saved:** Simplified the dependency graph and reduced code cleverness.
