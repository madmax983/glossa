## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** Manual implementations of `Drop`, `Clone`, and `PartialEq` on `ast::Expr` using `stacker::maybe_grow`.
**Cut:** Removed the manual implementations and added `#[derive(Clone, PartialEq)]`. Also removed tests that were asserting stack overflows in these AST structures, relying on the parser limit instead.
**Saved:** ~180 LOC and significant cognitive load. YAGNI on speculative generality as recursion depth is checked before AST creation.
