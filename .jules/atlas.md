## [Breaking The Knot: Semantic-IR Cycle]
**Tangle:** The `semantic` module imports `ir` to use `IteratorOp` and `lower_expr` for detecting iterator patterns. The `ir` module imports `semantic` to consume `AnalyzedProgram` for lowering. This creates a circular dependency `semantic <-> ir`.
**Blueprint:**
1. Define `AnalyzedIteratorOp` in `semantic` to mirror `IteratorOp` but with `AnalyzedExpr`.
2. Update `AnalyzedExpr` to use `AnalyzedIteratorOp`.
3. Stop calling `ir::lower_expr` inside `semantic`.
4. Update `ir` to map `AnalyzedIteratorOp` to `ir::IteratorOp` during lowering.
**Stability:** Decouples semantic analysis from IR generation details. `semantic` becomes strictly upstream of `ir`.

## [Splitting The Blob: Semantic Module]
**Tangle:** `src/semantic/mod.rs` had grown to over 4600 lines, mixing high-level orchestration, control flow parsing, pattern matching, and expression analysis. This violated the Single Responsibility Principle and made navigation difficult.
**Blueprint:** Split `src/semantic/mod.rs` into `control_flow.rs`, `declarations.rs`, `expressions.rs`, `patterns.rs`, and `conversion.rs`. `mod.rs` now acts as an orchestrator and facade.

## [Breaking The Knot: Semantic Types and Models]
**Tangle:** `src/semantic/types.rs` depended on `AnalyzedStatement` (from `mod.rs`) for method bodies, while `mod.rs` depended on `GlossaType` (from `types.rs`). This created a circular dependency where the Type System depended on the AST it was supposed to type.
**Blueprint:**
1. Created `src/semantic/model.rs` to hold all AST nodes (`AnalyzedStatement`, `AnalyzedExpr`) and Semantic Models (`TraitDef`, `TraitImpl`).
2. Moved `TraitDef` and friends from `types.rs` to `model.rs`.
3. Moved AST nodes from `mod.rs` to `model.rs`.
4. Refactored `mod.rs` to re-export `model` contents.
5. Removed legacy `SemanticAnalyzer` code that was duplicating logic.
**Stability:** Strictly separates Data (Model), Types (Type System), and Logic (Analysis). `model.rs` depends on `types.rs`, but `types.rs` is now leaf-level with no dependencies on the AST.
