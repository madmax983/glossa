The only lines left are `119` in `src/tools/mosaic.rs` which is `crate::ast::Statement::TestDeclaration(_) => "Test Declaration",`
Wait! `crate::ast::Statement::TestDeclaration` in the `match stmt_other` block inside `run_mosaic_inner`.
My test constructs a `stmt_test` manually and does `match &stmt_test`, but it does NOT call `run_mosaic_inner("δοκιμή «τ» ...")`.
To properly cover line 119 in `run_mosaic_inner`, I need to pass a string that parses as `TestDeclaration` into `run_mosaic_inner`.
There is already a test string:
```rust
        let source = "
            ὁ μέγας καὶ καλὸς ἄνθρωπος τὸν λόγον λέγει.
            τοῦ πατρὸς τοῦ θεοῦ ὁ λόγος.
            ὁ ἄνθρωπος ὁ ἰδὼν καὶ ἀκούσας λέγει.
            γ μῆκος μέγεθος λέγε.

            // Other decls
            χαρακτήρ Τ ὁρίζειν { }.
            εἶδος Χ τῷ Τ ἐμπίπτειν { }.
            δοκιμή «τ» .
                1 1 ἰσοῦται.
            τέλος.
        ";
```
Wait! `test_mosaic` has this string! And it DOES call `run_mosaic_inner`. Why is line 119 not covered?
Let's see the `parse` code. Does "δοκιμή «τ» . 1 1 ἰσοῦται. τέλος." parse as a `TestDeclaration`?
Let's check parser syntax for tests:
```glossa
δοκιμή «test name».
    ξ 5 ἰσοῦται.
τέλος.
```
In `test_mosaic`, the string is:
```glossa
            δοκιμή «τ» .
                1 1 ἰσοῦται.
            τέλος.
```
But the assertion in `test_mosaic` is:
```rust
        assert!(output.contains("Trait Definition"));
        assert!(output.contains("Trait Implementation"));
        assert!(output.contains("Test Declaration"));
```
Wait, if the assert passes, `output.contains("Test Declaration")` is true! Why does llvm-cov say line 119 is missed?
Wait, the `run_mosaic_inner` iterates over statements.
Line 119 is exactly `crate::ast::Statement::TestDeclaration(_) => "Test Declaration",`.
Let's check `cargo llvm-cov` output for `mosaic.rs` line 119 again:
It missed line 119!
Maybe `output.contains("Test Declaration")` is passing because something else emits it?
No, nothing else emits it. Wait.
