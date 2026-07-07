**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored Haruspex visit_expr and visit_statement**
**Learning:** Found two massive match statements (`visit_expr` at 165 lines and `visit_statement` at 109 lines) handling AST to Graphviz DOT conversion. The huge matches made it difficult to follow the main structure.
**Action:** Flattened both functions by extracting simpler arms into `visit_literal_expr`, `visit_wrapper_expr`, `visit_binding_statement`, and `visit_expr_list_statement` helpers, improving readability without changing logic.
