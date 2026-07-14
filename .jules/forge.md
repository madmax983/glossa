**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored Tester's print_test_results**
**Learning:** Found a god function handling multiple logical components like headers, success/failure summary, results table, and error details output in one monolithic block.
**Action:** Used guard clauses and extracted discrete functionality (`print_tester_header`, `print_status_banner`, `print_results_table`, `print_failure_details`) from `print_test_results` to vastly improve readability and lower cognitive overhead.
