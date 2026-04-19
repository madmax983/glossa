**[Boilerplate in CLI Tools]**
**Learning:** Many CLI tools (weave, labyrinth, cartographer, alchemist, tester) duplicate the exact same boilerplate for parsing and analyzing source code, including manual `match` statements for error handling. `runner.rs` has a private `analyze_source` helper that handles this cleanly.
**Action:** Make `analyze_source` public in `runner.rs` (or move it to a shared `utils.rs` if needed) and replace the duplicated `parse`/`analyze_program` boilerplate across all tools.

**[Labyrinth CFG Builder]**
**Learning:** The `build_statement` function in `labyrinth.rs` became a "God Object" by attempting to inline the mermaid.js graph node construction for all 17 AST statement variants into a single massive `match` block.
**Action:** Always extract the internal logic of complex `match` arms (like recursive branches in `If`, `While`, `For`, `Match`) into their own private helper functions. This turns the main `match` block into a clean router and drastically reduces cognitive load.

**[Codegen Statement Generator]**
**Learning:** The `generate_statement` function in `src/codegen.rs` became a "God Object" by inlining the translation logic for all 17 AST statement variants into a single massive `match` block.
**Action:** Always extract the internal logic of complex `match` arms (like `If`, `While`, `For`, `Match`) into their own private helper functions. This turns the main `match` block into a clean router and drastically reduces cognitive load.
