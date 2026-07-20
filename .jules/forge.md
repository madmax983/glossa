**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactoring highlight_expr and print_morphological_analyses**
**Learning:** Found deeply nested logic and extremely long match arms (God Functions) in `highlight_expr` (`src/tools/highlight.rs`) and `print_morphological_analyses` (`src/tools/dictionary.rs`).
**Action:** Flattened and extracted these massive inline segments into separate, well-named helper functions (`highlight_unary_op`, `highlight_block`, `highlight_array_literal` and `format_morphological_grammar`). This drastically improves readability while retaining zero-cost behavior.
