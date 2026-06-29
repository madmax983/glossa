**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored CLI main.rs**
**Learning:** Found a god object function > 190 lines handling the matching and execution of all CLI commands.
**Action:** Extracted the massive `match cli.command` block into a clear `execute_command(cli: Cli)` helper function to flatten the file structure and improve readability.
