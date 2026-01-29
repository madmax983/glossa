## 2024-05-22 - Code Gen Monoliths
**Learning:** `src/codegen/rust.rs` relied on massive `match` statements handling every HIR variant in a single function (`generate_statement`, `generate_expr`). This makes the code hard to read and navigate.
**Action:** Extract logic for each enum variant into private helper functions (e.g., `generate_let`, `generate_if`) to flatten the structure and improve readability.
