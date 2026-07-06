**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored Scholar's run_scholar**
**Learning:** Found a large monolithic function `run_scholar` that mixed file I/O with multiple loops iterating over different scopes to generate formatted documentation, violating separation of concerns.
**Action:** Extracted the documentation formatting loops into separate focused helper functions (`format_types`, `format_traits`, `format_functions`) using `Iterator<Item = ...>` to allow passing `.peekable()` iterators without requiring allocations.
