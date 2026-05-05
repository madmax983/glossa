use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]
    #[test]
    #[ignore]
    fn test_havoc_semantic_overflow_proptest(_seed in 0..1) {
        let mut expr = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        };

        for _ in 0..100_000 {
            expr = AnalyzedExpr {
                expr: AnalyzedExprKind::PropertyAccess {
                    owner: Box::new(expr),
                    property: "prop".into(),
                },
                glossa_type: GlossaType::Number,
            };
        }

        // Dropping this deeply nested expression will cause a stack overflow!
        drop(expr);
    }
}
