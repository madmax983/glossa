**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored Haruspex visit_expr and visit_statement**
**Learning:** Found two god object functions > 100 lines handling massive match arms for AST nodes.
**Action:** Created clear, small helpers (`visit_literal_expr`, `visit_wrapper_expr`, `visit_assert_expr`, `visit_simple_statement`) to flatten the match statements and dispatcher logic.

**Refactored patterns.rs try_parse_struct_instantiation**
**Learning:** Found a god object function > 100 lines handling phrase structure validation, collection type checking, and struct type checking.
**Action:** Created clear, small helpers (`extract_instantiation_components`, `handle_collection_instantiation`) and utilized guard clauses to early-return and flatten the structure.
