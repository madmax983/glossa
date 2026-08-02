**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored Semantic loop variable extraction**
**Learning:** `parse_for_range_loop` and `parse_for_iteration_loop` both contained a duplicated nested `if let` block to extract the loop variable name.
**Action:** Extracted this into a private helper function `extract_loop_variable(body_clauses: &[Clause], default_name: &str) -> smol_str::SmolStr`.
