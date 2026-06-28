# 👺 Havoc: Stack Overflow in AST Drop via Proptest

🧨 **The Trigger:**
Generating deeply nested semantic AST nodes (`AnalyzedExpr`) using `proptest` successfully triggers a fatal stack overflow. The default `Drop` implementation for `AnalyzedExprKind::BinOp` recursively calls `Drop` on its boxed children. When the depth reaches the system's limit (found dynamically by `proptest`), it blows the stack.

📉 **The Stack Trace:**
```text
thread 'test_havoc_ast_drop_stack_overflow' has overflowed its stack
fatal runtime error: stack overflow, aborting
error: test failed, to rerun pass `--test havoc_proptest_deep_ast`
```

🧪 **Reproduction:**
Run `cargo test --test havoc_proptest_deep_ast`.

😈 **Comment:**
You assumed your trees would always be shallow because of parser limits. You forgot that ASTs can be manipulated or constructed programmatically (e.g., during macro expansion or aggressive optimizations). You were wrong.
