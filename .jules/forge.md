**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Flattening deeply nested loops**
**Learning:** Parsing loops often suffer from deep nesting due to `if/else` handling state (like strings) wrapping a large `match` block.
**Action:** Use guard clauses (`if state { ... continue; }`) at the top of the loop to handle specific states, allowing the main logic and `match` statements to remain un-indented and readable.
