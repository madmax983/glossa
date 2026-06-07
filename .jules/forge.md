**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored `main` and `tester` logic**
**Learning:** `main` was a single function of nearly 200 lines matching all CLI commands. `tester::print_test_results` was a 100+ line function doing several UI formatting tasks.
**Action:** Used macro/feature-gated helpers to decouple CLI match arms in `main`. Extracted reporting sections into clean helpers (`print_test_header`, `print_success_summary`, `print_failure_summary`, `print_results_table`, `print_failure_details`) in `tester.rs`.
