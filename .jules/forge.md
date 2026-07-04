**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Extracted long formatting and AST visiting functions**
**Learning:** Found god object functions `GlossaReport::fmt` and `haruspex::visit_expr`. The `fmt` method contained extensive logic for constructing both metrics and functions tables. `visit_expr` handled everything including repetitive wrapper elements (`Some`, `Ok`, `None`).
**Action:** Created clear, small helpers (`format_metrics_table`, `format_functions_table` for report.rs; `visit_literal_expr`, `visit_wrapper_expr` for haruspex.rs). Kept behavior exactly the same.
