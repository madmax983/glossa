use glossa::ast::Expr;

#[test]
fn test_debug_stack_overflow() {
    let mut expr = Expr::NumberLiteral(1);
    for _ in 0..20000 {
        expr = Expr::Phrase(vec![expr]);
    }
    let _debug_str = format!("{:?}", expr);
}
