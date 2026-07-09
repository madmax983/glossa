**The Target:** Rust system crash / panics.
**The Weak Point:** Fuzzing with arbitrary structure nesting and malformed UTF-8.
**The Trigger:** A generated syntax tree hitting extreme recursion limits during parsing, analysis, cloning, and dropping; or malicious `&str` inputs to morphological layers.
**The Wreckage:**
1. `havoc_proptest_limits` testing > 500,000 AST nodes deep gracefully halted.
2. `cargo-fuzz` survived `126,000` iterations directly into FFI/Morphology boundaries without a single panic.
3. `havoc_dos` verified `/dev/zero` infinite stream aborts safely with no OOM.
4. Attempted stack exhaustion on clone/drop of `Program` / `Statement` handled safely by `stacker`.
5. **[Deep Expression Stack Overflow]**
**Trigger:** `AnalyzedExprKind` with deeply nested `BinOp` bypassing parser limits programmatically.
**Action:** Created `tests/havoc_codegen_stack_overflow.rs` to detonate this vulnerability via stack overflow during code generation.
