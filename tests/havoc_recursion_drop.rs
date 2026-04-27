#![allow(missing_docs)]
use glossa::ast::{BinOperator, Clause, Expr, Statement, UnaryOperator, Word};

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
