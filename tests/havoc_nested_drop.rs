use proptest::prelude::*;
use glossa::semantic::*;

proptest! {
    // We don't want a ton of runs because it crashes the test runner completely.
    // Proptest is used here to prove we found a generic structural vulnerability.
    #![proptest_config(ProptestConfig::with_cases(1))]
    #[test]
    fn test_ast_drop_stack_overflow(_ in ".*") {
        let mut expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: GlossaType::Boolean,
        };

        // Nest deeply enough to blow the stack when the compiler automatically derives Drop
        for _ in 0..100000 {
            expr = AnalyzedExpr {
                expr: AnalyzedExprKind::Some(Box::new(expr)),
                glossa_type: GlossaType::Boolean,
            };
        }

        // Explicitly letting it drop at the end of the scope
    }
}
