**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored Tester's print_test_results**
**Learning:** Found a god object function > 110 lines that handled multiple distinct rendering steps for test outputs.
**Action:** Created clear, small helpers (`print_overall_status_header`, `print_test_cases_table`, `print_failure_details`) and passed the appropriate variables down.
