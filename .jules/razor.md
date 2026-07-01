## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `GnomonVisitor` and `AuditorVisitor` used unnecessary single-use object-oriented object abstractions in `src/tools/gnomon.rs` and `src/tools/auditor.rs` to keep track of simple state while traversing AST statements.
**Cut:** Flattened both visitors into procedural `visit_statement` functions by passing mutable state references (`&mut current_depth` and `&mut max_depth` for `gnomon`, and `&mut FxHashMap` and `&mut FxHashSet` for `auditor`).
**Saved:** Removed two unused visitor structs, reducing boilerplate in two analysis tools while matching the procedural refactor direction.
