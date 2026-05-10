**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored Codegen Quote Blocks**
**Learning:** Found deep nesting ('Pyramid of Doom') in `generate_fn_def`, `generate_trait_def`, `generate_trait_impl`, and `generate_if` within `src/codegen.rs` due to `match` statements and duplicated `quote!` blocks handling optional values (like `return_type` or `else_body`).
**Action:** Replaced `match` with `if let` guard clauses and extracted optional TokenStream components (like `ret_ty_tokens`) to flatten the logic and avoid duplicating `quote!` structures, adhering closer to idiomatic Rust.
