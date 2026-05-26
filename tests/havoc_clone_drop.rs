use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};

#[test]
fn test_havoc_semantic_drop_stack_overflow() {
    let mut expr = AnalyzedExpr {
        expr: AnalyzedExprKind::BooleanLiteral(true),
        glossa_type: GlossaType::Boolean,
    };
    // Deep recursion
    for _ in 0..100_000 {
        expr = AnalyzedExpr {
            expr: AnalyzedExprKind::PropertyAccess {
                owner: Box::new(expr),
                property: "a".into(),
            },
            glossa_type: GlossaType::Boolean,
        };
    }
    // When expr is dropped at the end of the scope, the implicit recursive
    // Drop implementation of `AnalyzedExpr` will overflow the stack.
}
