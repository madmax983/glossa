**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored Tester's print_test_results**
**Learning:** Found a god function `print_test_results` doing too much: formatting the header, success summary, failure summary, results table, and error extraction.
**Action:** Extracted logic into `print_header`, `print_summary_status`, `print_results_table`, and `print_failures`.
