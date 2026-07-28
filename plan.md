We can see `test_parse_match_pattern_uncovered` tests `Expr::Phrase` with `ἄλλο`, `δύο`, and undefined word `ἀγνώστου`. It also tests `Expr::Word` with undefined word `ἀγνώστου`.
It does NOT test:
- `Expr::Word` wildcard ("αλλο")
- `Expr::Word` numeral ("δύο")
- `Expr::Word` variable (defined)
- `Expr::Phrase` variable (defined)
- `Expr::Word` function (defined)
- `Expr::Phrase` function (defined)

Wait, if we merged the logic into a helper function, Codecov tracks diff coverage. Since `analyze_match_word` handles both now, if some parts of the function are hit by `Expr::Phrase` but not `Expr::Word`, does it lower the coverage? Maybe the `analyze_match_word` is called twice in the diff (once from `Phrase`, once from `Word`) and both calls are hit.
Wait, let's just add those missing coverage points to `test_parse_match_pattern_uncovered`.

```rust
        // Word with wildcard "ἄλλο"
        let word_wildcard = Expr::Word(Word::new("ἄλλο"));
        let res = parse_match_pattern(&word_wildcard, &mut scope).unwrap();
        if let AnalyzedExprKind::BooleanLiteral(b) = res.expr {
            assert!(b);
        } else {
            panic!("Expected boolean literal");
        }

        // Word with numeral "δύο"
        let word_numeral = Expr::Word(Word::new("δύο"));
        let res = parse_match_pattern(&word_numeral, &mut scope).unwrap();
        if let AnalyzedExprKind::NumberLiteral(n) = res.expr {
            assert_eq!(n, 2);
        } else {
            panic!("Expected number literal");
        }

        // Word with defined variable
        scope.define("ξ".to_string(), GlossaType::Number);
        let word_var = Expr::Word(Word::new("ξ"));
        let res = parse_match_pattern(&word_var, &mut scope).unwrap();
        if let AnalyzedExprKind::Variable(v) = res.expr {
            assert_eq!(v, "ξ");
        } else {
            panic!("Expected variable");
        }

        // Phrase with defined variable
        let phrase_var = Expr::Phrase(vec![Expr::Word(Word::new("ξ"))]);
        let res = parse_match_pattern(&phrase_var, &mut scope).unwrap();
        if let AnalyzedExprKind::Variable(v) = res.expr {
            assert_eq!(v, "ξ");
        } else {
            panic!("Expected variable");
        }
```
Let's add this to `test_parse_match_pattern_uncovered` to guarantee 100% diff hit.
