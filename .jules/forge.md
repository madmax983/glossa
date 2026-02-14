**Refactoring `analyze_argument_expr_recursive`**
**Learning:** Breaking down a large `match` statement into small, named helper functions significantly improves readability and makes the code more self-documenting. It also allows for easier testing of individual components if needed in the future.
**Action:** Always look for opportunities to extract logic from large `match` arms, especially when they contain nested logic or error handling. Use meaningful names for helper functions that describe the variant being handled (e.g., `analyze_array`, `analyze_block`).

**Refactoring `build_statement` in parser**
**Learning:** Pest parser AST construction often defaults to iterating over all children (`into_inner()`) and accumulating state. When the grammar guarantees mutually exclusive alternatives (e.g., `statement = { A | B | C }`), inspecting the first child (`pairs.next()`) allows for direct dispatch, eliminating mutable state variables and loops.
**Action:** Review grammar rules for "choice" patterns and refactor corresponding builder functions to use dispatch logic instead of accumulation loops.
