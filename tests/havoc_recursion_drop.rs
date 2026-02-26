use glossa::ast::{BinOperator, Clause, Expr, Statement, UnaryOperator, Word};

#[test]
fn test_deep_recursion_phrase() {
    // Depth: 100,000
    // We use a slightly smaller depth for this test to ensure it runs quickly in CI,
    // but still enough to overflow a default stack if unprotected.
    // 20,000 is usually enough to overflow 2MB stack.
    let depth = 20_000;

    // Construct deep expression: Phrase([Phrase([Phrase(...)])])
    let mut deep_expr = Expr::NumberLiteral(1);
    for _ in 0..depth {
        deep_expr = Expr::Phrase(vec![deep_expr]);
    }

    // 1. Test Clone (should stack overflow if recursive)
    println!("Cloning deep expression (depth {})...", depth);
    let cloned = deep_expr.clone();

    // 2. Test Drop (should stack overflow if recursive)
    println!("Dropping cloned expression...");
    drop(cloned);

    println!("Dropping original expression...");
    drop(deep_expr);

    println!("Success!");
}

#[test]
fn test_deep_recursion_comparison() {
    let depth = 20_000;

    // Construct two identical deep expressions
    let mut expr1 = Expr::NumberLiteral(1);
    for _ in 0..depth {
        expr1 = Expr::Phrase(vec![expr1]);
    }

    let expr2 = expr1.clone();

    // 3. Test PartialEq (should stack overflow if recursive)
    println!("Comparing deep expressions...");
    assert_eq!(expr1, expr2);

    // Construct a different deep expression
    let mut expr3 = Expr::NumberLiteral(2); // Difference at leaf
    for _ in 0..depth {
        expr3 = Expr::Phrase(vec![expr3]);
    }

    assert_ne!(expr1, expr3);
}

#[test]
fn test_expr_variants_coverage() {
    // This test ensures that Clone, Drop, and PartialEq logic for ALL variants
    // is exercised, boosting code coverage for the manual implementations.

    let w1 = Word::new("test");
    let _w2 = Word::new("test");
    let exprs = vec![
        Expr::StringLiteral("s".to_string()),
        Expr::NumberLiteral(42),
        Expr::BooleanLiteral(true),
        Expr::ArrayLiteral(vec![Expr::NumberLiteral(1), Expr::NumberLiteral(2)]),
        Expr::IndexAccess {
            array: Box::new(Expr::ArrayLiteral(vec![])),
            index: Box::new(Expr::NumberLiteral(0)),
        },
        Expr::Word(w1),
        Expr::Phrase(vec![Expr::NumberLiteral(1)]),
        Expr::PropertyAccess {
            owner: Box::new(Expr::Word(Word::new("owner"))),
            property: Box::new(Expr::Word(Word::new("prop"))),
        },
        Expr::Call {
            verb: Word::new("call"),
            arguments: vec![Expr::NumberLiteral(1)],
        },
        Expr::Binding {
            name: Word::new("var"),
            value: Box::new(Expr::NumberLiteral(10)),
        },
        Expr::BinOp {
            left: Box::new(Expr::NumberLiteral(1)),
            op: BinOperator::Add,
            right: Box::new(Expr::NumberLiteral(2)),
        },
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            operand: Box::new(Expr::BooleanLiteral(true)),
        },
        Expr::Block(vec![Statement::Regular {
            clauses: vec![Clause {
                expressions: vec![Expr::NumberLiteral(1)],
            }],
            is_query: false,
            is_propagate: false,
        }]),
    ];

    for expr in exprs {
        // Test Clone
        let cloned = expr.clone();

        // Test PartialEq (True)
        assert_eq!(expr, cloned);

        // Test PartialEq (False) - simple check against a different variant
        let diff = Expr::NumberLiteral(999);
        if let Expr::NumberLiteral(_) = expr {
            // Skip if it matches the diff type but has different value (already covered by != check usually)
            // But here we just want to ensure the _ => false arm or mismatch arms are hit.
        } else {
            assert_ne!(expr, diff);
        }

        // Test Drop (implicit)
    }
}
