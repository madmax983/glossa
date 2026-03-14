use glossa::ast::{Clause, Expr, Program, Statement, Word};
use glossa::semantic::analyze_program;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_deep_ast_no_overflow(
        depth in 100..5000usize
    ) {
        let mut expr = Expr::Word(Word::new("root"));
        for _ in 0..depth {
            expr = Expr::PropertyAccess {
                owner: Box::new(expr),
                property: Box::new(Expr::Word(Word::new("prop"))),
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
}

proptest! {
    #[test]
    fn test_deep_ast_index_access_no_overflow(
        depth in 100..5000usize
    ) {
        let mut expr = Expr::Word(Word::new("root"));
        for _ in 0..depth {
            expr = Expr::IndexAccess {
                array: Box::new(expr),
                index: Box::new(Expr::Word(Word::new("prop"))),
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
}
