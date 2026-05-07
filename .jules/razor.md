## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** Unnecessary deep folder hierarchy `src/semantic/assembly/mod.rs` and `src/semantic/assembly/model.rs` with low file count. And `src/errors/mod.rs` and `src/errors/assembly.rs`.
**Cut:** Flattened `src/semantic/assembly/model.rs` into `src/semantic/assembly.rs`. Flattened `src/errors/assembly.rs` into `src/errors/mod.rs`. Deleted the directories and sub-files.
**Saved:** 2 module files and 1 directory removed.
