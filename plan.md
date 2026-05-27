1. **Analyze The Blob**: `src/semantic/conversion.rs` is a huge file (~2880 lines) responsible for converting `AssembledStatement` into `AnalyzedStatement`. This breaks the single responsibility principle and makes the module hard to navigate.
2. **Action: Architect**: Split `src/semantic/conversion.rs` into a new `src/semantic/conversion/` module structure:
   - `mod.rs`: Contains the main entry point `convert_assembled_to_analyzed` and `classify_assembled_statement`.
   - `bindings.rs`: For `classify_variable_binding` and `classify_assignment`.
   - `collections.rs`: For collection mutations (`classify_collection_mutation`, `classify_insert`, etc.).
   - `functions.rs`: For function calls and method resolution (`classify_function_call`, `classify_genitive_method_call`, etc.).
   - `assertions.rs`: For `classify_assertion` and `classify_equality_assertion`.
   - `prints.rs`: For print and query statements (`classify_print`, `classify_query`, etc.).
   - `values.rs`: For `extract_value` and its myriad helper functions (`extract_literal`, `extract_binary_op`, etc.).
3. **Verify**:
   - Run `cargo check`.
   - Run `cargo test`.
   - Format with `cargo fmt`.
   - Make sure no external public APIs are broken.
4. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
