**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored Mosaic's run_mosaic_inner**
**Learning:** Found a large `for` loop body in `run_mosaic_inner` handling assembly logic and formatting rows for different statement types.
**Action:** Extracted the logic into a private helper function `add_statement_row`, returning the nested block back to the first level, adhering to the 'flatten structure' and 'extract god functions' principles.
