**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**[Refactoring parse_struct_args in src/semantic/patterns.rs]
**Learning:** Found a function `parse_struct_args` containing a manual `for` loop with a large inner `match` statement. Manual looping with accumulator vectors in such parsing functions obscures the functional data flow.
**Action:** Extract the complex inner `match` statement into a smaller, named helper function (`parse_single_struct_arg`), and replace the manual loop with an iterator pipeline (`.iter().enumerate().filter_map().collect()`).
