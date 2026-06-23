## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## Remove AuditorVisitor and GnomonVisitor Abstract Classes
**Bloat:** Using a Visitor abstract class pattern to calculate simple metrics like finding maximum loop depth in AST tree or doing static checking.
**Cut:** Wrote flat, procedural functions that traverse the tree recursively in a single file instead of maintaining a separate Visitor object. Also reduced object instantiation overhead.
**Saved:** About 100 lines of boilerplate Object-Oriented code in `src/tools/gnomon.rs` and `src/tools/auditor.rs`.
