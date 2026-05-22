## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** Boilerplate variable bindings for conditionally compiled match arms in CLI (binding variables and then ignoring them).
**Cut:** Utilize struct wildcards `{ .. }` in the fallback arm to cleanly ignore all bound variables while handling the `cfg(not(feature = "nova"))` case cleanly without unused variable lints.
**Saved:** Suppressed `unused_variables` warnings from rustc while cleaning up the CLI matching code.
