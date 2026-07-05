**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored Haruspex God Functions**
**Learning:** `visit_statement` and `visit_expr` in `haruspex.rs` were >100 line God functions utilizing massive match blocks for every AST node type.
**Action:** Extracted the match arms into domain-specific helpers (`visit_simple_statement`, `visit_literal_expr`, etc.) to flatten the hierarchy and reduce cognitive load.
