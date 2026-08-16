**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**[Refactoring God Functions with Arrays of Function Pointers]**
**Learning:** Long chains of `if let Some(res) = func(...) { return ... }` (common in AST evaluation and semantic analysis) create repetitive boilerplate that is hard to read.
**Action:** Extract inline logic into named helpers and use an array of function pointers (`let extractors = [func1, func2]; for ext in extractors { ... }`) to flatten the structure and create a clean "Chain of Responsibility".
