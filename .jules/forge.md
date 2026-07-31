**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored Tester's print_test_results**
**Learning:** Found a god object function > 100 lines handling test summary, details table rendering, and failure extraction logic all inline.
**Action:** Created clear, small helpers (`print_test_summary_banner`, `print_test_results_table`, `print_test_failure_details`) and passed the required subsets of state to each.
