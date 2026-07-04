1. Refactor `visit_expr` in `src/tools/haruspex.rs` to extract literal handling and wrapper handling (like `Some`, `None`, `Ok`, `Err`, `Unwrap`, `Try`) into helper functions (`visit_literal_expr` and `visit_wrapper_expr`) to reduce the length of `visit_expr` below 50 lines.
2. Refactor `fmt` function in `src/tools/report.rs` (GlossaReport and CompilationReport) to extract formatting into separate helper methods (`format_metrics_table`, `format_functions_table`, etc.) to shorten `fmt` functions.
3. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
4. Open a PR with the required format for the "Forge" persona.
