Ah, I see. I added the manual branch matching but that's in tests, which is just testing the SAME LOGIC, it's not actually running line 119 in `run_mosaic_inner`.
The actual line 119 is inside `run_mosaic_inner`.

To hit it, I should just make the test run `run_mosaic_inner` on a real source containing `TestDeclaration`.

```rust
        let source = "
            δοκιμή «test» .
                ξ 5 ἰσοῦται.
            τέλος.
        ";
        let mut buffer = Vec::new();
        run_mosaic_inner(source, &mut buffer).unwrap();
        assert!(String::from_utf8(buffer).unwrap().contains("Test Declaration"));
```
This is because `TestDeclaration` is parsed but the string in `test_mosaic` was:
```glossa
            δοκιμή «τ» .
                1 1 ἰσοῦται.
            τέλος.
```
Wait, the string in `test_mosaic` ALREADY HAS `δοκιμή «τ» .` but it's not being matched. Why?
Let's see the parser output! `cargo run --features nova -- check <<<'δοκιμή «τ» . 1 1 ἰσοῦται. τέλος.'`
