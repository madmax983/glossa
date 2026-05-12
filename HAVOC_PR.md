# 👺 Havoc: Semantic Fuzzing & AnalyzedExpr Clone/Drop Stack Overflow

🧨 **The Trigger:**
Passing a deeply recursive string into the Codegen phase or dropping/cloning heavily nested `AnalyzedExpr` nodes bypasses the strict recursion bounds built into the initial AST parser. `fuzz_target_codegen` quickly triggered an out-of-memory or stack overflow crash when `AnalyzedExpr` attempted to tear down the recursively constructed semantic structures without `stacker::maybe_grow` protecting the drop.

📉 **The Stack Trace:**
```text
thread 'test_havoc_analyzedexpr_drop' has overflowed its stack
fatal runtime error: stack overflow, aborting
```

🧪 **Reproduction:**
- **Fuzzing:** Run `cargo +nightly fuzz run fuzz_target_codegen -- -max_total_time=10`
- **Tests:** Run `cargo test --test havoc_clone_drop`

😈 **Comment:**
Warden put armor on the parser AST but left the semantic models naked and exposed to implicit recursive drops. Never leave `Clone` or `Drop` naked on recursive trees. I have added `fuzz_target_codegen` to hammer the parser-to-codegen pipeline, and `havoc_clone_drop.rs` safely proves the crash without stalling `cargo test`.
