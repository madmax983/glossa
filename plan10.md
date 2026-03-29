In PR 806 I see 85.71% of diff hit for `codecov/patch`.
Wait! The files modified in `mosaic-ui-polish` are:
`src/tools/mosaic.rs`
The lines I added are:
```rust
                    Err(e) => {
                        table.add_row(vec![
                            Cell::new(format!("{}", i + 1)),
                            Cell::new(format!(
                                "\u{3a3}\u{3c6}\u{3ac}\u{3bb}\u{3bc}\u{3b1} (Error): {}",
                                e
                            ))
                            .fg(Color::Red),
                            Cell::new(""),
                            Cell::new(""),
                            Cell::new(""),
                            Cell::new(""),
                        ]);
                    }
```
Wait! I modified 14 lines in the Err block!
Did I add tests to hit the Err block?
Yes:
```rust
        let source = "ὁ ἄνθρωπος λέγει γράφει."; // Double verb triggers an AssemblyError directly
        let mut buffer = Vec::new();
        let _ = run_mosaic_inner(source, &mut buffer);
        let output = String::from_utf8(buffer).unwrap();

        assert!(
            output.contains("\u{3a3}\u{3c6}\u{3ac}\u{3bb}\u{3bc}\u{3b1} (Error): "),
            "Output should contain localized Error string"
        );
```
So these lines ARE hit!
Wait! The GitHub Action failed check is NOT about my test breaking.
It is: `85.71% of diff hit (target 93.51%)`.
Where is the diff missing coverage?
In the `cargo llvm-cov` output, it says:
`Uncovered Lines: -- /app/src/tools/mosaic.rs: 119`
Line 119 is:
`crate::ast::Statement::TestDeclaration(_) => "Test Declaration",`
Is it the ONLY uncovered line?
Yes! `Uncovered Lines: /app/src/tools/mosaic.rs: 119`
I added a manual test that constructs an AST, but I removed it!
Wait! I replaced it with `run_mosaic_inner("δοκιμή «test» . 1 1 ἰσοῦται. τέλος.", &mut buffer)`.
But it did NOT hit line 119!
WHY?
Because `run_mosaic_inner` uses `parse(source)?`.
Does `δοκιμή «test» . 1 1 ἰσοῦται. τέλος.` successfully parse as `TestDeclaration`?
Let's see what `run_mosaic_inner` does. It iterates over statements in `program.statements`.
Wait! Does `δοκιμή «test» . 1 1 ἰσοῦται. τέλος.` parse into `TestDeclaration`?
Let's see the AST!
