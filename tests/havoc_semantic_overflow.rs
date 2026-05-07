use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};

#[test]
#[ignore]
fn test_havoc_semantic_overflow() {
    let mut expr = AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(1),
        glossa_type: GlossaType::Number,
    };

    // Deeply nest the expression to trigger a stack overflow upon drop
    for _ in 0..1_000_000 {
        expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BinOp {
                left: Box::new(expr),
                op: glossa::morphology::BinaryOp::Add,
                right: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
            },
            glossa_type: GlossaType::Number,
        };
    }

    // Attempt to drop the deeply nested expression, which will stack overflow
    drop(expr);
}
