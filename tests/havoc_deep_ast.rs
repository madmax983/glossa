#![allow(missing_docs)]
use glossa::ast::{Clause, Expr, Program, Statement};

#[test]
#[ignore = "Demonstrates a SIGABRT stack overflow when directly analyzing deeply nested ASTs bypassing parser limits"]
fn test_deep_ast_overflow_analyzer() {
    let depth = 50000;

    // We construct a deeply nested AST manually.
    // This simulates an API consumer constructing their own AST, bypassing the parser's
    // string length or `check_recursion_depth` constraints, or an unbounded macro expansion.
    let mut expr = Expr::NumberLiteral(1);
    for _ in 0..depth {
        expr = Expr::Phrase(vec![expr]);
    }

    let stmt = Statement::Regular {
        clauses: vec![Clause {
            expressions: vec![expr],
        }],
        is_query: false,
        is_propagate: false,
    };

    let prog = Program {
        statements: vec![stmt],
    };

    // 💥 DETONATE: This will cause a fatal stack overflow (SIGABRT)
    let _res = glossa::semantic::analyzer::analyze_program(&prog);
}
