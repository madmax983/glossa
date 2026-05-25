use glossa::semantic::{AnalyzedStatement, AnalyzedExpr, AnalyzedExprKind, GlossaType};

#[test]
fn wreckage_analyzed_stmt_clone_drop() {
    let mut stmt = AnalyzedStatement::Break;
    for _ in 0..100000 {
        stmt = AnalyzedStatement::While {
            condition: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: GlossaType::Boolean,
            }),
            body: vec![stmt],
        };
    }

    let cloned = stmt.clone();
    drop(stmt);
    drop(cloned);
}
