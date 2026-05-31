## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** Ignoring undefined names in object fallback and allowing arbitrary double subjects for print verbs.
**Cut:** Modified object fallback to correctly return UndefinedName for undefined variables. Removed the broad print verb exception for double subjects while allowing it specifically for array filtering. Tightened verb presence checks to emit MissingVerb errors instead of allowing them to proceed to codegen and fail as an ICE.
**Saved:** Multiple lines of confusing behavior and ICE crashes avoided.
