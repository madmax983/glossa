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

**[Tester God Function Refactor]**
**Learning:** `extract_failures` in `src/tools/tester.rs` was a deeply nested function spanning ~75 lines to parse compiler output, hiding multiple concerns (skipping sections, parsing names, capturing multi-line messages, cleaning panic noise).
**Action:** Extract logic into dedicated helpers (`parse_failure_name`, `capture_failure_message`, `clean_panic_message`) using early returns/guard clauses. This flattens the loop from 4-level deep nesting into a simple orchestrator.

**[Haruspex AST Node God Functions]**
**Learning:** `visit_statement` and `visit_expr` inside `DotGenerator` in `src/tools/haruspex.rs` grew exceptionally long (both exceeding 200 lines) by attempting to handle every AST match arm natively within the match block. This obfuscated graph emission logic. When using scripts to refactor GraphViz DOT generators, one must be exceedingly careful about string formatting and escaping `\n`.
**Action:** Extract the complex `match` branches (e.g., `AnalyzedStatement::If`, `AnalyzedExprKind::FunctionCall`) into individual helper methods (`visit_if_statement`, `visit_function_call_expr`, etc.), leaving the central match blocks as clean router functions. Always ensure `.dot` format literals retain `\\n`.

**[Tester God Function Refactor]**
**Learning:** `print_test_results` in `src/tools/tester.rs` was a large "God Function" (100 lines) that handled multiple distinct concerns: formatting the header, building the summary table, and parsing/printing detailed failure outputs.
**Action:** Extract these distinct logical blocks into dedicated private helpers (`print_report_header`, `print_results_table`, `print_failure_details`) to flatten the main function and improve readability.
