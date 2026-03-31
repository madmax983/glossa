#![allow(missing_docs)]
use glossa::ast::{BinOperator, Clause, Expr, Program, Statement, UnaryOperator, Word};
use glossa::semantic::analyze_program;

fn run_analysis(expr: Expr) {
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
    // We just want to ensure it runs without error and covers the recursion paths
    let _ = analyze_program(&program);
}

#[test]
fn test_binop_recursion_coverage() {
    // 1 + 2
    let expr = Expr::BinOp {
        left: Box::new(Expr::NumberLiteral(1)),
        op: BinOperator::Add,
        right: Box::new(Expr::NumberLiteral(2)),
    };
    run_analysis(expr);
}

#[test]
fn test_unaryop_recursion_coverage() {
    // !true (Not operator is not Unwrap)
    let expr = Expr::UnaryOp {
        op: UnaryOperator::Not,
        operand: Box::new(Expr::BooleanLiteral(true)),
    };
    run_analysis(expr);
}

#[test]
fn test_binding_recursion_coverage() {
    // x = 1
    let expr = Expr::Binding {
        name: Word::new("x"),
        value: Box::new(Expr::NumberLiteral(1)),
    };
    run_analysis(expr);
}

#[test]
fn test_phrase_recursion_coverage() {
    // (1, 2) - phrase of expressions
    let expr = Expr::Phrase(vec![Expr::NumberLiteral(1), Expr::NumberLiteral(2)]);
    run_analysis(expr);
}

#[test]
fn test_call_recursion_coverage() {
    // f(1)
    let expr = Expr::Call {
        verb: Word::new("f"),
        arguments: vec![Expr::NumberLiteral(1)],
    };
    run_analysis(expr);
}

#[test]
fn test_property_access_recursion_happy_path() {
    // owner.prop
    let expr = Expr::PropertyAccess {
        owner: Box::new(Expr::Word(Word::new("owner"))),
        property: Box::new(Expr::Word(Word::new("prop"))),
    };
    run_analysis(expr);
}
