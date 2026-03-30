use glossa::ast::*;
use glossa::semantic::analyzer::analyze_program;

#[test]
#[ignore = "Demonstrates a SIGABRT stack overflow when directly analyzing deeply nested ASTs bypassing parser limits"]
fn test_analyzer_stack_overflow() {
    let depth = 50000;
    let mut expr = Expr::Word(Word::new("root"));
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

    let program = Program {
        statements: vec![stmt],
    };

    // 💥 DETONATE: This will cause a fatal stack overflow (SIGABRT)
    let _ = analyze_program(&program);
}
