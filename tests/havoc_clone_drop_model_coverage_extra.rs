use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
use glossa::semantic::CaptureMode;

#[test]
fn test_analyzed_expr_kind_clone_drop_coverage_extra() {
    // Just clone the exact missing one PropertyAccess
    let expr = AnalyzedExpr {
        expr: AnalyzedExprKind::PropertyAccess {
            owner: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit }),
            property: "prop".into(),
        },
        glossa_type: GlossaType::Number,
    };

    let cloned = expr.clone();
    drop(cloned);
    drop(expr);
}
