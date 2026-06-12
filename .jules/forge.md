**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored `main.rs` Feature Boilerplate**
**Learning:** Found massive repetitive boilerplate for experimental `nova` CLI commands causing a 200+ line `main` function.
**Action:** Extracted the conditional compilation and error handling into a concise `macro_rules! run_nova_cmd`.
