## 2024-05-22 - Code Gen Monoliths
**Learning:** `src/codegen/rust.rs` relied on massive `match` statements handling every HIR variant in a single function (`generate_statement`, `generate_expr`). This makes the code hard to read and navigate.
**Action:** Extract logic for each enum variant into private helper functions (e.g., `generate_let`, `generate_if`) to flatten the structure and improve readability.

## 2024-05-23 - Assembler Feed Monolith
**Learning:** `src/semantic/assembler.rs` contained a `feed` method that mixed low-level token checks (operators, markers) with high-level grammatical routing. This made the function difficult to follow and violated the single responsibility principle.
**Action:** Extracted logic for special markers, method verbs, operators, and properties into dedicated helper methods (`check_special_markers`, `check_method_verbs`, etc.) to create a cleaner, high-level dispatch in `feed`.

## 2024-05-24 - Semantic Conversion Monolith
**Learning:** `src/semantic/conversion.rs` contained `classify_assembled_statement`, a 600+ line function responsible for interpreting all assembled statement types. This "God Function" was difficult to maintain and reason about.
**Action:** Extracted 11 helper functions (e.g., `classify_iterator_pattern`, `classify_struct_instantiation`) to handle specific patterns, and replaced the main function body with a sequential dispatch sequence. Also refactored `extract_value` to reduce duplication.
