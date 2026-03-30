use glossa::ast::{Expr, Statement, Clause, Word};
use glossa::semantic::analyzer::analyze_program;
use glossa::ast::Program;

#[test]
fn test_stack_overflow() {
    let depth = 50000;

    let mut expr = Expr::Phrase(vec![Expr::Word(Word::new("test"))]);
    for _ in 0..depth {
        expr = Expr::Phrase(vec![expr]);
    }

    let program = Program {
        statements: vec![Statement::Regular {
            clauses: vec![Clause {
                expressions: vec![expr],
            }],
            is_query: false,
            is_propagate: false,
        }],
    };

    let _ = analyze_program(&program);
}
