**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored Tester's print_test_results**
**Learning:** Found a god object function > 100 lines handling multiple distinct CLI output tasks (header, summary banner, results table, failure details).
**Action:** Extracted these distinct UI output blocks into separate small helper functions (`print_header`, `print_summary`, `print_results_table`, `print_failure_details`) to make the main function purely declarative.
