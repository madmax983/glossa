**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored Tester's print_test_results**
**Learning:** Found a god function > 110 lines long handling table creation, logic dispatching, extraction, terminal rendering, and nested loops, making understanding the test summary output logic very difficult.
**Action:** Applied the 'Extract' and 'Flatten' maneuvers to break it apart into focused, small helper functions (`print_summary_banner`, `print_results_table`, `print_failures_section`), leaving behind a clear coordinator.
