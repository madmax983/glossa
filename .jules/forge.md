**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored expressions.rs feed_expr_recursive**
**Learning:** Found a god function `feed_expr_recursive` in `src/semantic/expressions.rs` which was highly nested with match arms.
**Action:** Extracted the match arms into dedicated helper functions (`feed_phrase`, `feed_property_access`, `feed_call`, `feed_binding`, `feed_binop`, `feed_unary_op`), improving readability and reducing nesting.
**Refactored patterns.rs try_parse_struct_instantiation**
**Learning:** Found a large function `try_parse_struct_instantiation` in `src/semantic/patterns.rs` which was highly nested and hard to read.
**Action:** Refactored the function by extracting instantiation logic into dedicated helpers (`create_collection_instantiation`, `create_struct_instantiation`), flattening the structure, and applying early returns.
