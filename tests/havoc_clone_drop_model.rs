use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
use std::thread;

#[test]
fn test_analyzed_expr_clone_drop_stack_overflow() {
    let child = thread::Builder::new()
        .stack_size(1024 * 1024 * 10)
        .spawn(|| {
            let depth = 50000;
            let mut expr = AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            };

            for _ in 0..depth {
                expr = AnalyzedExpr {
                    expr: AnalyzedExprKind::Some(Box::new(expr)),
                    glossa_type: GlossaType::Number,
                };
            }

            let expr2 = expr.clone();
            drop(expr2);
            drop(expr);
        })
        .unwrap();
    child.join().unwrap();
}
