**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Extracted execute_command**
**Learning:** The CLI command routing logic in src/main.rs was heavily bloated by a massive match statement dealing with feature flags and different tool options, which obscured the flow of the main() function.
**Action:** Extracted the routing block into a dedicated execute_command helper function, preserving the original behavior while drastically flattening main().
**Extracted execute_command Coverage Fix**
**Learning:** Testing CLI command routing variants correctly using `cargo test` required bypassing the `Repl` block via `#[cfg(test)]` to prevent blocking on `stdin`. This exposed a secondary issue where the `run_repl` import became unused during tests.
**Action:** Guarded both the `run_repl` invocation and its corresponding `use` statement with appropriate `#[cfg]` attributes to satisfy both coverage metrics and strict linting.
