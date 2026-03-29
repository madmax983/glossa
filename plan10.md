Oh! Lines 464, 465, 466, 467 are:
```rust
455         // Force hitting the TestDeclaration branch inside run_mosaic_inner directly
456         let source_test = "δοκιμή «test» . 1 1 ἰσοῦται. τέλος.";
457         let mut buffer_test = Vec::new();
458         run_mosaic_inner(source_test, &mut buffer_test).unwrap();
459         let output_test = String::from_utf8(buffer_test).unwrap();
460         assert!(output_test.contains("Test Declaration"));
```
Wait. They are 455-460.
Line 464 is `let mut asm_ops = AssembledStatement::default();`
Why does `cargo llvm-cov` say lines 464-467 are missed?
Let me run it again.
