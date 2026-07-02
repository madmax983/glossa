**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored Scholar and CLI**
**Learning:** Found massive repetitive macro/cfg logic in `src/main.rs`, heavily duplicated struct instantiations in `src/morphology/disambiguation.rs`, and a god function in `src/tools/scholar.rs` combining file I/O with document formatting.
**Action:** Created `nova_command!` macro to centralize cfg toggling. Created `ctx` closure in `analyze_article` to reduce struct allocation verbosity. Extracted Markdown formatters into `format_types`, `format_traits`, `format_functions` to cleanly split concerns.
