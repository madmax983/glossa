## 2026-01-28 - Dismantling the Assembler God Object
**Tangle:** The `Assembler` struct in `src/semantic/assembler.rs` was a flat list of ~20 fields mixing sentence constituents (Subject/Verb/Object) with expression parsing state (Literals/Operators), making it a "God Object" that managed too many distinct parsing states in a single context.
**Blueprint:** Split `Assembler` into `SentenceState` (grammatical slots) and `ExpressionState` (syntactic buffer), composed into the main `Assembler` struct. This enforces separation of concerns between sentence structure and expression evaluation while maintaining the same public API.

## 2026-01-28 - [Breaking The Knot: Semantic-IR Cycle]
**Tangle:** The `semantic` module imports `ir` to use `IteratorOp` and `lower_expr` for detecting iterator patterns. The `ir` module imports `semantic` to consume `AnalyzedProgram` for lowering. This creates a circular dependency `semantic <-> ir`.
**Blueprint:**
1. Define `AnalyzedIteratorOp` in `semantic` to mirror `IteratorOp` but with `AnalyzedExpr`.
2. Update `AnalyzedExpr` to use `AnalyzedIteratorOp`.
3. Stop calling `ir::lower_expr` inside `semantic`.
4. Update `ir` to map `AnalyzedIteratorOp` to `ir::IteratorOp` during lowering.
**Stability:** Decouples semantic analysis from IR generation details. `semantic` becomes strictly upstream of `ir`.
