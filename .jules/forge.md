**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored `main.rs` main function**
**Learning:** Found a "God Function" `main` > 190 lines handling both argument parsing and CLI command matching with deep nested configurations for experimental tools.
**Action:** Extracted the massive `match cli.command` block into a standalone `handle_command` helper, keeping `main` strictly focused on high-level entry and file execution, and flattened `Commands::Gnomon` by prefixing unused variable `input`.
