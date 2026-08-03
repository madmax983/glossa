**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored classify_expression in semantic conversion**
**Learning:** Found a "God Function" `classify_expression` (~78 lines) handling literal building, expression fallback building, and propagation application.
**Action:** Extracted logic into `build_initial_expressions`, `build_fallback_binary_expression`, `build_subject_object_fallback`, and `apply_propagation`. Used `#[allow(clippy::collapsible_if)]` on outer loops to prevent Clippy from suggesting unstable `let_chains` syntax.
