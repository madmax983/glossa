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

## [Extracting The Model: Semantic Data Structures]
**Tangle:** `src/semantic/mod.rs` was still bloated with core data structure definitions. `src/semantic/types.rs` had a circular dependency with `src/semantic/mod.rs` because `TraitDef` (in `types.rs`) contained `AnalyzedStatement` (in `mod.rs`).
**Blueprint:**
1. Created `src/semantic/model.rs` to house the Semantic AST (`AnalyzedStatement`, `AnalyzedExpr`, etc.).
2. Moved `TraitDef` and `TraitImpl` from `types.rs` to `model.rs` to resolve the cycle.
3. Updated `mod.rs` to export `model`.
4. Removed legacy `SemanticAnalyzer` code.
**Stability:** Establishes a clean DAG: `types` (GlossaType) <- `model` (AnalyzedStatement) <- `resolver` (Scope) <- `program` (AnalyzedProgram).
