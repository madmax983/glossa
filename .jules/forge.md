**[Boilerplate in CLI Tools]**
**Learning:** Many CLI tools (weave, labyrinth, cartographer, alchemist, tester) duplicate the exact same boilerplate for parsing and analyzing source code, including manual `match` statements for error handling. `runner.rs` has a private `analyze_source` helper that handles this cleanly.
**Action:** Make `analyze_source` public in `runner.rs` (or move it to a shared `utils.rs` if needed) and replace the duplicated `parse`/`analyze_program` boilerplate across all tools.

**[Labyrinth CFG Builder]**
**Learning:** The `build_statement` function in `labyrinth.rs` became a "God Object" by attempting to inline the mermaid.js graph node construction for all 17 AST statement variants into a single massive `match` block.
**Action:** Always extract the internal logic of complex `match` arms (like recursive branches in `If`, `While`, `For`, `Match`) into their own private helper functions. This turns the main `match` block into a clean router and drastically reduces cognitive load.

**[Codegen Statement Generator]**
**Learning:** The `generate_statement` function in `src/codegen.rs` became a "God Object" by inlining the translation logic for all 17 AST statement variants into a single massive `match` block.
**Action:** Always extract the internal logic of complex `match` arms (like `If`, `While`, `For`, `Match`) into their own private helper functions. This turns the main `match` block into a clean router and drastically reduces cognitive load.

**[Auditor Visitor God Functions]**
**Learning:** The `visit_statement` and `visit_expr` functions in `src/tools/auditor.rs` became "God Functions" containing massive `match` blocks.
**Action:** Extract complex `match` arms for `If`, `While`, `For`, `Match`, and repetitive logic into private helper functions like `visit_if_statement` and `visit_exprs` to flatten nesting and clarify the match block routing.

**Extracting Prefix Validation to Flatten Logic**
**Learning:** Functions dealing with heavy pattern matching (like deep AST destructuring) can easily become "God Functions" with towering pyramids of doom or endless guard clauses. Breaking off the prefix-validation component into a separate tuple-returning helper function (`extract_..._prefix`) allows the main function to flatten the extraction into a single clean `if let Some(...)` destructuring binding.
**Action:** Always scan long functions for large blocks of early-return guard clauses that are just validating and extracting internal structure, and isolate them into a dedicated helper function that returns an `Option` containing the validated parts.

**Centralized Table Styling with `comfy_table`**
**Learning:** Manual border rendering using hardcoded ASCII strings (like `╭──────────────────╮`) mixed with `println!` loops is fragile and causes issues with multi-line outputs. It creates massive string constants and error-prone loop blocks.
**Action:** Replace manual terminal border strings with `comfy_table::Table` using `presets::UTF8_FULL`. This guarantees clean multi-line wrapping and robust ASCII styling without writing loops.
