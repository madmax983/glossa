👺 Havoc: Semantic AST Clone and Drop Stack Overflow

🧨 **The Trigger:** A deeply nested `AnalyzedExpr` tree constructed programmatically or bypassing limits causes a stack overflow when implicitly dropping or calling `.clone()` on it.

📉 **The Stack Trace:**
```
thread 'havoc_semantic_clone_drop_stack_overflow' (16750) has overflowed its stack
fatal runtime error: stack overflow, aborting
```

🧪 **Reproduction:** Run `HAVOC_DETONATE_SEMANTIC_OVERFLOW=1 cargo test --test havoc_semantic_stack_overflow -- havoc_semantic_clone_drop_stack_overflow --nocapture`.

😈 **Comment:** Warden meticulously secured the parser's AST but left the Semantic AST (`AnalyzedExpr` and `AnalyzedStatement`) completely exposed to stack overflows via derived `Clone` and implicit `Drop`. If I can crash it, I win.
