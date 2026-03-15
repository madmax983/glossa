use glossa::codegen::{generate_statement_code, to_rust_type};
use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Literal};

#[test]
fn force_index_overflow_panic() {
    let expr = AnalyzedExpr {
        kind: AnalyzedExprKind::IndexAccess {
            array: Box::new(AnalyzedExpr {
                kind: AnalyzedExprKind::ArrayLiteral(vec![]),
                glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
            }),
            index: Box::new(AnalyzedExpr {
                kind: AnalyzedExprKind::Literal(Literal::Number(i64::MAX)),
                glossa_type: GlossaType::Number,
            }),
        },
        glossa_type: GlossaType::Number,
    };

    // Generate code logic: this will literally write Rust code that panics at runtime if executed,
    // but the `generate_collection_index` macro just produces tokens. It doesn't panic during codegen.
}
