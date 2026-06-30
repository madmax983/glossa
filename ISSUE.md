**Title:** ⚒️ Forge: Refactor try_parse_struct_instantiation

**Description:**
🚮 **Smell:** `try_parse_struct_instantiation` in `src/semantic/patterns.rs` was a large function with deep nesting that was difficult to read and maintain.
✨ **Solution:** Extracted logic for creating collections and structs into dedicated helpers (`create_collection_instantiation`, `create_struct_instantiation`), flattened the structure with early returns, and decoupled struct field extraction logic from parsing logic.
🧼 **Benefit:** Dramatically reduces the cognitive load of reading the instantiation logic, strictly following Forge's rule of flattening nesting and separating logic.
🛡️ **Verification:** Tests passed. No logic changed.
