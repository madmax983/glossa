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

**Refactoring generate_statement in src/codegen.rs**
**Learning:** Extracting large match blocks that do not strictly require mutable state or complex logic into small, focused helper functions (like `generate_statement_binding`, `generate_statement_assignment`, etc.) significantly flattens the code structure and improves readability, adhering to the 'Grandma Test' and avoiding the 'Pyramid of Doom'.
**Action:** Regularly review large `match` blocks, especially those used for dispatch, and extract inline logic into well-named private functions.

**Refactoring `classify_*` in `src/semantic/conversion.rs`**
**Learning:** Functions that parse or classify elements using heuristic priorities often become a "Pyramid of Doom" through nested `if let Some(...) = ...` blocks. This increases cognitive load and hides the critical failure path.
**Action:** Use "Guard Clauses" (early returns) extensively in classification functions. `let Some(x) = y else { return ... };` keeps the execution path flat and adheres strictly to the 'Grandma Test'.

**2024-03-24 - Exhaustiveness Preservation**
**Learning:** Extracting large `match` statements by grouping variants under wildcard catch-alls (`_ => helper()`) destroys the compiler's exhaustiveness checking, leading to severe maintainability regressions if new variants are added.
**Action:** Always extract the *inner block logic* of specific match arms into helper functions while preserving the exhaustive structure of the outer `match` block.

**Refactoring God Functions in the Alchemist**
**Learning:** `transpile_statement` in `src/tools/alchemist.rs` was a classic God Function (over 200 lines) with a deeply nested match statement containing complex string building logic for each AST node. When using `replace_with_git_merge_diff` or python scripting to extract match arms into helper functions (`transpile_if`, `transpile_while`, etc.), it's critical to capture or destructure the enum variables correctly in the helper functions to prevent `unused_variables` warnings from clippy. In this case, I used `AnalyzedStatement::If { .. } => transpile_if(stmt, indent)` in the top-level match and handled destructuring inside `transpile_if` to resolve warnings.
**Action:** Always verify `cargo clippy --all-targets --all-features -- -D warnings` after extracting match arms. If variables bound in the match arm are no longer used locally because the whole object is passed, use `..` to ignore them.
**Refactoring `try_parse_trait_method_call` and `feed_expr_recursive`**
**Learning:** Refactoring deeply nested code using guard clauses correctly preserves readability and avoids "Pyramid of Doom" without altering logic. Similarly, extracting individual large arms of a large match statement into named helper functions directly tackles "God Object" smells and reduces cognitive load, without having to extract with wildcard catch-alls which bypasses exhaustiveness checks.
**Action:** Continually prioritize replacing nested `if let` blocks with early returns (guard clauses `let Some(x) = y else { return; }`) and decomposing large match statement inner logic into scoped helper functions to flatten nested code and improve readability.

**Refactoring God Functions in CLI Display Utilities**
**Learning:** Functions that generate complex UI tables (like `add_row` in `src/tools/mosaic.rs`) often become God Functions (~150+ lines) as they aggregate many different data types and conditions into a single string. This creates a "Pyramid of Doom" of data preparation right before UI construction.
**Action:** Extract the complex column data preparation logic into distinct formatting helper functions (e.g., `format_subject`, `format_other_column`), keeping the main row addition function focused solely on inserting the mapped columns into the table UI.

**Refactoring `classify_query`**
**Learning:** Functions evaluating containment and standard statements side-by-side often accumulate to 50+ lines due to nested query logic. Using guard clauses to immediately exit if the condition isn't met (`if !asm_stmt.is_query { return Ok(None); }`) removes one level of indentation. Furthermore, extracting dense logical blocks (like the containment condition) into dedicated helpers (`classify_containment_query`) flattens the execution path.
**Action:** When acting as Forge and encountering Pyramids of Doom, deeply nested logic, or long chained `if let` sequential returns, refactor by inverting the initial `if` statements into early returns (guard clauses) and extract large interior code blocks into highly readable, dedicated helper functions to flatten execution paths.

**Refactoring `parse_for_range_loop` and `process_participles`**
**Learning:** Extracting large, inline, nested logic blocks from `for` and `match` statements directly flattens execution paths and adheres to the 'Grandma Test'. Functions over 150 lines like `process_participles` become much easier to read when their main internal branches are extracted to specifically-named helper functions like `process_fold_participle` and `process_map_participle`.
**Action:** Continually prioritize replacing nested `if let` blocks with early returns and decomposing large looping constructs or match statement inner logic into scoped helper functions to flatten nested code and improve readability.
