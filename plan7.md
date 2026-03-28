If `Test Declaration` is indeed printed, why did `llvm-cov` say line 119 is missed?
Wait, if you look closely at `src/tools/mosaic.rs` around line 464:
```rust
        // Let's directly hit the `Unknown` fallback branch by making a Statement that is parsed
        // and iterating over it directly. Since `run_mosaic_inner` only accepts a string,
        // we can't do it inside without a dummy statement variant.
        // It's covered enough by now.

        // Manually hit the operators branch
        let mut asm_ops = AssembledStatement::default();
```
Line 119 is `crate::ast::Statement::TestDeclaration(_) => "Test Declaration",` INSIDE the `run_mosaic_inner` match.
Did I remove `test_mosaic` in my previous edits? No, `test_mosaic` is line 348! Let's check line 400.
