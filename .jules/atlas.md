## 2024-05-22 - Breaking the Semantic-IR Cycle and Taming the Assembler God Object
**Tangle:**
1. Circular Dependency: `src/semantic` depends on `src/ir` for `IteratorOp` and `lower_expr`, while `src/ir` depends on `src/semantic` for `AnalyzedExpr`. This violates the layered architecture where `semantic` should be upstream of `ir`.
2. God Object: `src/semantic/assembler.rs` manages state, token routing, validation, and type definitions, violating the Single Responsibility Principle.

**Blueprint:**
1. **Decouple Semantic from IR**: Introduce `AnalyzedIteratorOp` in `src/semantic/types.rs` so `semantic` no longer needs `ir::IteratorOp`. Remove calls to `lower_expr` within semantic analysis. Lowering to HIR will handle the conversion.
2. **Refactor Assembler**: Extract constituent data structures (`Constituent`, `AssembledStatement`, etc.) into a new module `src/semantic/constituents.rs`. This separates the *data* of the assembly from the *logic* of the assembler.
