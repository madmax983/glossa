**[Boilerplate in CLI Tools]**
**Learning:** Many CLI tools (weave, labyrinth, cartographer, alchemist, tester) duplicate the exact same boilerplate for parsing and analyzing source code, including manual `match` statements for error handling. `runner.rs` has a private `analyze_source` helper that handles this cleanly.
**Action:** Make `analyze_source` public in `runner.rs` (or move it to a shared `utils.rs` if needed) and replace the duplicated `parse`/`analyze_program` boilerplate across all tools.
