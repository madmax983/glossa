use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
use glossa::semantic::CaptureMode;

#[test]
fn test_analyzed_expr_kind_clone_drop_coverage() {
    let exprs = vec![
        AnalyzedExprKind::StringLiteral("a".into()),
        AnalyzedExprKind::NumberLiteral(1),
        AnalyzedExprKind::BooleanLiteral(true),
        AnalyzedExprKind::Variable("a".into()),
        AnalyzedExprKind::VerbCall { verb: "a".into(), args: vec![] },
        AnalyzedExprKind::BinOp { left: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit }), op: glossa::morphology::BinaryOp::Add, right: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit }) },
        AnalyzedExprKind::UnaryOp { op: glossa::morphology::UnaryOp::Not, operand: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit }) },
        AnalyzedExprKind::Range { start: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit }), end: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit }), inclusive: false },
        AnalyzedExprKind::ArrayLiteral(vec![]),
        AnalyzedExprKind::Some(Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit })),
        AnalyzedExprKind::Ok(Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit })),
        AnalyzedExprKind::Err(Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit })),
        AnalyzedExprKind::Unwrap(Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit })),
        AnalyzedExprKind::Try(Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit })),
        AnalyzedExprKind::IndexAccess { array: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit }), index: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit }) },
        AnalyzedExprKind::FunctionCall { func: "a".into(), args: vec![] },
        AnalyzedExprKind::MethodCall { receiver: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit }), method: "a".into(), args: vec![] },
        AnalyzedExprKind::StructInstantiation { type_name: "a".into(), fields: vec![], args: vec![] },
        AnalyzedExprKind::Lambda { params: vec![], body: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit }), capture_mode: CaptureMode::Borrow },
        AnalyzedExprKind::CollectionNew { collection_type: "List".to_string() },
        AnalyzedExprKind::Assert { condition: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit }) },
        AnalyzedExprKind::AssertEq { left: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit }), right: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unit }) },
    ];

    for expr_kind in exprs {
        let expr = AnalyzedExpr {
            expr: expr_kind,
            glossa_type: GlossaType::Number,
        };
        let cloned = expr.clone();
        drop(cloned);
        drop(expr);
    }
}
