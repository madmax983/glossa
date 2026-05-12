**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**[TokenStream generation extraction in codegen.rs]**
**Learning:** The 'quote!' macro in Rust can easily lead to a 'Pyramid of Doom' (nested if/else/match inside macro logic). I found duplicated structure logic being duplicated just to optionally add return types and other trailing tokens.
**Action:** Extract trailing optional parts to variables first using '.map' or '.then' so that the final 'quote!' macro doesn't branch on optionals. This flattened `generate_fn_def`, `generate_if`, `generate_let`, `generate_trait_def`, `generate_trait_impl`, and `generate_statement_return`.
