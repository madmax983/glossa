**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactoring God Functions**
**Learning:** Extracting parts of a massive formatting function into cleanly named helper functions (e.g. `print_summary_table`, `print_results_table`, `print_failure_details`) radically improves readability while preserving original functionality and safely decoupling distinct logical blocks.
**Action:** When a method primarily coordinates multiple sub-tasks (like different parts of an output string/table), break it down into specialized helpers. This flattens the pyramid of doom and makes testing or modifying individual format sections much simpler later.
