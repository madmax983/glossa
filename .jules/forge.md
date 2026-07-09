**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored Tester's print_test_results**
**Learning:** Found a god object function > 100 lines handling multiple distinct concerns: formatting success messages, formatting test rows, and extracting failure details.
**Action:** Extracted logic into `print_summary_banner`, `print_test_rows`, and `print_failure_details` to reduce cognitive load and simplify the structure.
