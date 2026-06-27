👺 Havoc: Codegen Stack Overflow via proc_macro2

🧨 **The Trigger:**
A deeply nested AST expression (depth 50,000) generated programmatically causes a stack overflow during code generation. Specifically, the `output.to_string()` call on `TokenStream` within `generate_rust_file` crashes internally within the `proc_macro2` crate's stringification logic because `stacker::maybe_grow` does not protect the internals of external macro crates.

📉 **The Stack Trace:**
```
thread 'havoc_codegen_stack_overflow' has overflowed its stack
fatal runtime error: stack overflow, aborting
```

🧪 **Reproduction:**
Run `cargo test --test havoc_codegen_stack_overflow`. It will spawn a subprocess that detonates by building an AST of depth 50,000 and attempting to generate Rust code for it.

😈 **Comment:**
You assumed that wrapping your own AST traversal in `stacker::maybe_grow` would save you. You forgot that `proc_macro2` has its own recursive stringification logic that you cannot control. You were wrong.

---

👺 Havoc: Semantic AST Clone & Drop Stack Overflow

🧨 **The Trigger:**
A deeply nested Semantic AST expression (depth 50,000) causes a stack overflow when cloned or dropped. While the `ast.rs` parser types use `stacker::maybe_grow`, the Semantic AST types (`AnalyzedExpr`, `AnalyzedStatement`) rely on derived `Clone` and implicit `Drop`, providing no recursion limit protections.

📉 **The Stack Trace:**
```
thread 'havoc_semantic_clone_drop_stack_overflow' has overflowed its stack
fatal runtime error: stack overflow, aborting
```

🧪 **Reproduction:**
Run `cargo test --test havoc_semantic_stack_overflow`. It will spawn a subprocess that detonates by building a Semantic AST of depth 50,000 and then attempting to clone and drop it.

😈 **Comment:**
You guarded the front door (parser) but left the back door wide open. You assumed `stacker::maybe_grow` on the initial AST would magically protect your derived `Clone` on an entirely different recursive data structure. You were wrong.
