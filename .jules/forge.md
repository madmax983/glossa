**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Eliminated Pyramid of Doom in Haruspex**
**Learning:** Found god functions `visit_expr` (165 lines) and `visit_statement` (109 lines) in `src/tools/haruspex.rs` that mapped large `match` arms for every AST node directly.
**Action:** Created small typed helper functions (`visit_wrapper_expr`, `visit_leaf_expr`, `visit_statement_list`) to extract and reuse graphviz DOT generation logic, dramatically flattening and shortening the god functions.
