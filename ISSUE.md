# 👺 Havoc: Code Generation Stack Overflow

🧨 **The Trigger:** Deeply nested AST expressions (depth 50,000) cause a stack overflow during code generation (`TokenStream::to_string()`) because `proc_macro2` and `quote` expansion is not protected against stack exhaustion.

📉 **The Stack Trace:**
```
thread 'havoc_codegen_ast_depth' (29477) has overflowed its stack
fatal runtime error: stack overflow, aborting
error: test failed, to rerun pass `--test havoc_codegen_depth`

Caused by:
  process didn't exit successfully: `/app/target/debug/deps/havoc_codegen_depth-50cde98849670e7f` (signal: 6, SIGABRT: process abort signal)
```

🧪 **Reproduction:** Run `cargo test --test havoc_codegen_depth`

😈 **Comment:** You assumed the AST would never be deeper than the stack limit. You were wrong.
