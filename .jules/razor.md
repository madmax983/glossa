## [Reduction]
**Bloat:** `GnomonVisitor` and `AuditorVisitor` stateful visitor structs, which were mostly just carrying state that could be passed explicitly, or carrying no state in the case of `GnomonVisitor`.
**Cut:** Flattened both struct-based visitors into pure procedural recursive functions (`max_loop_depth`, `visit_statement`, `visit_expr`) directly passing the state map context `&mut AuditorState`. Removed the `GnomonVisitor` completely and the single `AuditorVisitor` implementation logic was moved into modular functions.
**Saved:** Flattened unnecessary abstraction of visitor objects for simple traversal, eliminating boilerplates and reducing codebase size and cognitive load.
