## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## Trait Reduction
**Bloat:** Trait definitions (`χαρακτήρ`), implementations (`ἐμπίπτειν`), and associated methods (`AnalyzedMethod`, `TraitDef`, `TraitImpl`) created unnecessary abstraction layers and complex resolution logic.
**Cut:** Completely removed Trait variants from the AST (`Statement`), Semantic models (`AnalyzedStatement`), Parser logic, Resolver Scope, Codegen, and all CLI tools.
**Saved:** Hundreds of lines of parsing, resolving, code generation, and tooling traversal logic along with deep Trait lookup complexity.
