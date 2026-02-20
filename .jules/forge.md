**Refactoring `analyze_argument_expr_recursive`**
**Learning:** Breaking down a large `match` statement into small, named helper functions significantly improves readability and makes the code more self-documenting. It also allows for easier testing of individual components if needed in the future.
**Action:** Always look for opportunities to extract logic from large `match` arms, especially when they contain nested logic or error handling. Use meaningful names for helper functions that describe the variant being handled (e.g., `analyze_array`, `analyze_block`).

**Refactoring `build_statement` in parser**
**Learning:** Pest parser AST construction often defaults to iterating over all children (`into_inner()`) and accumulating state. When the grammar guarantees mutually exclusive alternatives (e.g., `statement = { A | B | C }`), inspecting the first child (`pairs.next()`) allows for direct dispatch, eliminating mutable state variables and loops.
**Action:** Review grammar rules for "choice" patterns and refactor corresponding builder functions to use dispatch logic instead of accumulation loops.

**Refactoring `runner.rs` tool orchestration**
**Learning:** CLI tools often repeat validation and setup logic (file existence, size checks, cache management) across multiple commands. Extracting these into helper functions (`load_source`, `analyze_source`) and dedicated modules (`cache.rs`) clarifies the core intent of each command.
**Action:** Identify repeated "pre-flight" checks in command handlers and unify them into a single pipeline or context struct early.

**Refactoring `src/parser.rs`**
**Learning:** Moving complex validation logic like recursion depth checking into its own module (`parser/recursion.rs`) keeps the main parser file focused on AST construction.
**Action:** Identify large, standalone validation functions and extract them to dedicated modules, even if they are only used in one place, to reduce cognitive load in the main file.

**Refactoring `generate_expr` in `src/codegen/mod.rs`**
**Learning:** Grouping helper functions by category (Simple, Complex, Control Flow) makes the main dispatch function (`generate_expr`) trivial to read and significantly reduces cognitive load when navigating the file.
**Action:** When extracting logic from large match statements, group the extracted helpers into logical sections within the file to maintain navigability.
