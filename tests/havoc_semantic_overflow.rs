use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
use smol_str::SmolStr;

#[test]
fn test_semantic_stack_overflow() {
    // 🧨 The Trigger: Deeply nested AnalyzedExpr triggering AddressSanitizer or stack overflow on drop
    let mut expr = AnalyzedExpr {
        expr: AnalyzedExprKind::None,
        glossa_type: GlossaType::Unknown,
    };

    for _ in 0..50000 {
        expr = AnalyzedExpr {
            expr: AnalyzedExprKind::PropertyAccess {
                owner: Box::new(expr),
                property: SmolStr::new("prop"),
            },
            glossa_type: GlossaType::Unknown,
        };
    }

    // Dropping this deeply nested structure will blow the stack
    drop(expr);
}
