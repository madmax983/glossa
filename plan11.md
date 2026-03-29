Wait! It DOES output `Test Declaration`.
So the arm `TestDeclaration(_) => "Test Declaration"` IS being executed!
BUT `cargo llvm-cov` says it is NOT?
Why does `cargo llvm-cov` say it is NOT?
Let's see what `cargo llvm-cov` output said.
```
tools/mosaic.rs                      558                 8    98.57%          16                 0   100.00%         334                 2    99.40%           0                 0         -
/app/src/tools/mosaic.rs: 119
```
Wait! `cargo llvm-cov --all-features` runs `cargo test`.
My manual `TestDeclaration` string is in `test_mosaic_error_and_missing_subject`:

```rust
        // Force hitting the TestDeclaration branch inside run_mosaic_inner directly
        let source_test = "δοκιμή «test» . 1 1 ἰσοῦται. τέλος.";
        let mut buffer_test = Vec::new();
        run_mosaic_inner(source_test, &mut buffer_test).unwrap();
        let output_test = String::from_utf8(buffer_test).unwrap();
        assert!(output_test.contains("Test Declaration"));
```

Does this fail to hit line 119? Let's check `test_run.gl` output or the log from my `cargo test --features nova`.
Yes! `test_mosaic_error_and_missing_subject` passed!
If it passed, `output_test.contains("Test Declaration")` was true!
If `output_test` contained `"Test Declaration"`, how could line 119 NOT be executed?!

Ah! The `Type Definition` string is in TWO places in `mosaic.rs`!
```rust
            stmt_other => {
                // For non-regular statements, just print the type
                let type_name = match stmt_other {
                    crate::ast::Statement::TypeDefinition(_) => "Type Definition",
                    crate::ast::Statement::TraitDefinition(_) => "Trait Definition",
                    crate::ast::Statement::TraitImpl(_) => "Trait Implementation",
                    crate::ast::Statement::TestDeclaration(_) => "Test Declaration", // LINE 119
                    crate::ast::Statement::Regular { .. } => unreachable!(),
                };
```
Is there ANOTHER `Test Declaration` somewhere?
Let's check `mosaic.rs` line 120 vs 200.
