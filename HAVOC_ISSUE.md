# 👺 Havoc: Stack Overflow in Semantic AST (Clone/Drop)

🧨 **The Trigger:**
Programmatically constructing a deeply nested `AnalyzedExpr` AST node (depth > 50,000) and allowing it to be implicitly dropped, cloned via the derived `Clone` implementation, or validated. While the parser's syntax AST (`Expr`) is protected by `stacker::maybe_grow`, the Semantic AST (`AnalyzedExpr` and `AnalyzedStatement`) remains completely unprotected, leading to an immediate stack overflow when traversing the recursive enums during deallocation.

📉 **The Stack Trace:**
```
thread 'havoc_semantic_clone_drop_stack_overflow' has overflowed its stack
fatal runtime error: stack overflow, aborting
error: test failed, to rerun pass `-p glossa --test havoc_semantic_stack_overflow`
```

🧪 **Reproduction:**
Run `cargo test --features nova -p glossa --test havoc_semantic_stack_overflow -- --ignored --nocapture`

😈 **Comment:**
You armored the front gates (Parser AST) but left the inner keep (Semantic AST) wide open. I bypassed your linear depth scanner and crafted a recursive payload directly. Your default `Drop` and derived `Clone` implementations simply walk off the edge of the stack into the abyss.
