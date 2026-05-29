**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored `try_parse_struct_instantiation`**
**Learning:** Found a god object function (`try_parse_struct_instantiation`) in `src/semantic/patterns.rs` handling AST destructuring, struct pattern validation, and different instantiation paths.
**Action:** Created clear, small helpers (`extract_single_phrase`, `validate_struct_pattern`, `build_collection_instantiation`, `build_user_struct_instantiation`) and used them in both `try_parse_struct_instantiation` and `try_parse_method_call` to flatten nesting and remove duplication.
