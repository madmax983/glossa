## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.
## [Reduction]
**Bloat:** `TraitMethodParts` factory struct in `src/codegen.rs` serving merely to return three distinct values.
**Cut:** Flattened by removing the struct entirely and just returning a standard tuple `(Ident, Vec<TokenStream>, Option<TokenStream>)` from `generate_trait_method_parts`.
**Saved:** 5 LOC and reduced cognitive load by removing an unnecessary abstraction.
