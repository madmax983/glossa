**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored Tester's print_test_results**
**Learning:** Found a god function (> 110 lines) handling header formatting, success/failure summaries, test result tables, and parsing/printing error details.
**Action:** Extracted it into four smaller helper functions (`print_header`, `print_summary`, `print_results_table`, `print_failures`) to improve readability and separation of concerns.
