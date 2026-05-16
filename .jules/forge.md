**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Haruspex Refactor**
**Learning:** Python scripts used for text replacement can accidentally swallow double backslashes in strings (`\\n`, `\\"`), causing bugs in raw string output like Graphviz DOT files.
**Action:** When using Python scripts to apply text-based refactoring to Rust code, ensure that backslashes in Rust string macros are correctly escaped.
