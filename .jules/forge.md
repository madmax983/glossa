**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored Tester's print_test_results**
**Learning:** Found a god function handling multiple logical chunks of console formatting (header, status, table, errors) making it >100 lines.
**Action:** Extracted into `print_test_results_header`, `print_overall_status`, `print_results_table`, and `print_failure_details`.
