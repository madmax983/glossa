use glossa::ast::{Expr, Program, Statement, Clause};
use glossa::semantic::analyze_program;

#[test]
fn test_exploit_ast_overflow() {
    let mut ast_expr = Expr::NumberLiteral(1);

    // Create nested layers
    for _ in 0..50000 {
        ast_expr = Expr::Phrase(vec![ast_expr]);
    }

    let stmt = Statement::Regular {
        clauses: vec![
            Clause {
                expressions: vec![ast_expr],
            }
        ],
        is_query: false,
        is_propagate: false,
    };

    let program = Program {
        statements: vec![stmt],
    };

    let result = analyze_program(&program);
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("Recursion limit exceeded"),
        "Error was: {}",
        err_str
    );
}
