🗑️ **Smell**: The `fmt` method in `src/tools/report.rs` and the `visit_expr` method in `src/tools/haruspex.rs` had evolved into "Pyramid of Doom" style God functions, each being well over 140 lines long and containing large nested `match` statements mixed with multi-line layout formatting.

✨ **Solution**:
* In `src/tools/report.rs`, the generation logic for both the global metrics table and the function definitions table were extracted from `fmt` into targeted helper methods: `format_metrics_table(&self)` and `format_functions_table(&self)`.
* In `src/tools/haruspex.rs`, the massive 165+ line `match` block in `visit_expr` was severely flattened. Similar unary and binary structural elements were unified, and literal value match arms were collapsed to cleanly execute `emit_node` calls using early bindings.

🧹 **Benefit**: Flattening the scope significantly improves readability and halves the visual footprint of the targeted functions. Extracted table generation functions provide logical separation of concerns without impacting runtime behavior.

🛡️ **Verification**: All `cargo test` checks passed locally. Validated generated output formatting via existing string matching tests.
