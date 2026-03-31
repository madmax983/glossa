#![allow(missing_docs)]
use glossa::ast::{Clause, Expr, Program, Statement, Word};
use glossa::semantic::analyze_program;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_ast_semantic_massive_chain_no_panic(
        depth in 100..500usize
    ) {
        let mut expr = Expr::NumberLiteral(1);
        for _ in 0..depth {
            expr = Expr::BinOp {
                left: Box::new(expr),
                op: glossa::ast::BinOperator::Add,
                right: Box::new(Expr::NumberLiteral(1)),
            };
        }
        let stmt = Statement::Regular {
            clauses: vec![Clause {
                expressions: vec![expr],
            }],
            is_query: false,
            is_propagate: false,
        };

        let program = Program {
            statements: vec![stmt],
        };
        let _ = analyze_program(&program);
    }

    #[test]
    fn test_ast_eq_massive_chain_no_panic(
        depth in 100..5000usize
    ) {
        let mut expr = Expr::NumberLiteral(1);
        for _ in 0..depth {
            expr = Expr::BinOp {
                left: Box::new(expr),
                op: glossa::ast::BinOperator::Add,
                right: Box::new(Expr::NumberLiteral(1)),
            };
        }
        let expr2 = expr.clone();
        assert_eq!(expr, expr2);
    }

    #[test]
    fn test_ast_clone_no_overflow(
        depth in 100..5000usize
    ) {
        let mut expr = Expr::Word(Word::new("root"));
        for _ in 0..depth {
            expr = Expr::IndexAccess {
                array: Box::new(expr),
                index: Box::new(Expr::Word(Word::new("prop"))),
            };
        }
        let expr2 = expr.clone();
        assert_eq!(expr, expr2);
    }
}
