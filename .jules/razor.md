## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** [The over-engineered pattern] Missing verb check relying on deep unwrap that panicked rather than throwing a MissingVerb semantic error, as well as a silent failure on undefined variables producing empty strings. We removed the unwrap.
**Cut:** [The simplified solution] Surgically checked explicitly on `!assembled.is_query` right after `assemble_statement` and after `convert_assembled_to_analyzed` to explicitly check exactly the edge cases `havoc_issue_echo` surfaces.
**Saved:** [Lines of code / Cognitive load] Kept all other features working perfectly by using `is_some_and` to exact words rather than completely revamping statement traits or lambda traits which broke 40+ tests earlier.
