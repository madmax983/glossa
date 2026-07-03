**The Target:** Rust system crash / panics.
**The Weak Point:** `proc_macro2::TokenStream` recursion inside `codegen.rs`.
**The Trigger:** A deeply nested semantic `AnalyzedExpr` tree (e.g. depth > 50_000) causes stack overflow when `generate_rust` attempts to convert the deeply nested `TokenStream` into a String.
**The Wreckage:**
`fatal runtime error: stack overflow, aborting`

Reproduction: `cargo test --test havoc_codegen_stack_overflow`

Comment: You assumed the TokenStream could safely hold infinitely deep expressions before formatting. You were wrong.
