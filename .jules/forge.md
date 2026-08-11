**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Extracted execute_command**
**Learning:** The CLI command routing logic in src/main.rs was heavily bloated by a massive match statement dealing with feature flags and different tool options, which obscured the flow of the main() function.
**Action:** Extracted the routing block into a dedicated execute_command helper function, preserving the original behavior while drastically flattening main().
