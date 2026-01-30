## 2024-05-22 - Code Gen Monoliths
**Learning:** `src/codegen/rust.rs` relied on massive `match` statements handling every HIR variant in a single function (`generate_statement`, `generate_expr`). This makes the code hard to read and navigate.
**Action:** Extract logic for each enum variant into private helper functions (e.g., `generate_let`, `generate_if`) to flatten the structure and improve readability.

## 2024-05-23 - Assembler Feed Monolith
**Learning:** `src/semantic/assembler.rs` contained a `feed` method that mixed low-level token checks (operators, markers) with high-level grammatical routing. This made the function difficult to follow and violated the single responsibility principle.
**Action:** Extracted logic for special markers, method verbs, operators, and properties into dedicated helper methods (`check_special_markers`, `check_method_verbs`, etc.) to create a cleaner, high-level dispatch in `feed`.

## 2024-05-24 - Semantic Conversion Monolith
**Learning:** `src/semantic/conversion.rs`'s `classify_assembled_statement` was handling every possible statement type in a single function, leading to deep nesting and poor readability.
**Action:** Extracted each statement pattern (iterator, property access, struct instantiation, etc.) into its own `detect_*` helper function returning `Option`. This allows the main function to simply chain checks.
