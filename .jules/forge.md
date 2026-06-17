**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored codegen generate_fn_def to Context Struct**
**Learning:** Found a function `generate_fn_def` with 4 parameters in `src/codegen.rs`.
**Action:** Extracted the parameters into `FnDefCtx<'_>` struct to improve readability.
