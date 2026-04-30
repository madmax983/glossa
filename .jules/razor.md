## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** Complex `is_defined()` state-machine checks spread across fallback AST extraction routines (like `try_print_default`) to catch undefined variables, causing cascading failures in method parsing (`self`).
**Cut:** Left the fallback tolerance intact where it resolves valid trait methods. Handled missing verbs and double subjects simply by removing arbitrary `is_print_verb` exception checks in the semantic assembler.
**Saved:** Prevented introducing a massive web of exceptions for `self` and struct properties just to catch undefined variables in the fallback parser. Simple changes in assembly handled the bulk of the Havoc assertions.
