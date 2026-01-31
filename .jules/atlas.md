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

## [Decoupling Semantic Model]
**Tangle:** The `semantic` module suffered from poor separation of concerns. Data structures like `AnalyzedProgram` and `TraitDef` were scattered between `mod.rs` and `types.rs`. `mod.rs` contained legacy analysis logic (`SemanticAnalyzer`) mixed with module orchestration. `types.rs` contained both Type System definitions (`GlossaType`) and AST nodes (`TraitDef`), creating implicit circular dependencies.
**Blueprint:**
1. Created `src/semantic/model.rs` to house all Semantic AST nodes (`AnalyzedStatement`, `TraitDef`, etc.).
2. Purified `src/semantic/types.rs` to contain only Type System definitions (`GlossaType`).
3. Cleaned `src/semantic/mod.rs` by removing legacy code and re-exporting the model.
**Stability:** Separates "Data" (model) from "Logic" (analysis) and "Types" (type system). Eliminates potential circular dependencies and makes the module structure intuitive.
