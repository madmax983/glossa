**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored Haruspex's visit_expr and visit_statement**
**Learning:** Found two God Objects (`visit_expr` > 160 lines and `visit_statement` > 100 lines) with massive match statements making the logic unreadable.
**Action:** Flattened the nested match blocks by extracting identical logic handling similar complex nodes into helpers like `visit_literal_expr`, `visit_wrapper_expr`, `visit_assignment_binding`, and `visit_multi_expr_statement`.
