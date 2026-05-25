use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};

#[test]
fn wreckage_analyzed_expr_clone_drop() {
    let mut expr = AnalyzedExpr {
        expr: AnalyzedExprKind::BooleanLiteral(true),
        glossa_type: GlossaType::Boolean,
    };

    for _ in 0..100000 {
        expr = AnalyzedExpr {
            expr: AnalyzedExprKind::Unwrap(Box::new(expr)),
            glossa_type: GlossaType::Boolean,
        };
    }

    let cloned = expr.clone();
    drop(expr);
    drop(cloned);
}
