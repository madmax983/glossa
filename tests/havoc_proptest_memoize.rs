use glossa::codegen::generate_rust;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, CaptureMode, GlossaType,
    Scope,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_memoize_arguments_panic(
        num_args in 1..10usize
    ) {
        let body = AnalyzedExpr {
            expr: AnalyzedExprKind::Variable("x".into()),
            glossa_type: GlossaType::Number,
        };

        let mut params = Vec::new();
        let mut param_types = Vec::new();
        for i in 0..num_args {
            params.push(format!("arg{}", i).into());
            param_types.push(GlossaType::Number);
        }

        let lambda = AnalyzedExpr {
            expr: AnalyzedExprKind::Lambda {
                params,
                body: Box::new(body),
                capture_mode: CaptureMode::Memoize,
            },
            glossa_type: GlossaType::Function {
                params: param_types,
                returns: Box::new(GlossaType::Number),
            },
        };

        let stmt = AnalyzedStatement::Expression(vec![lambda]);

        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope: Scope::new(),
        };

        // This should panic because we pass arguments to a Memoize closure
        let result = std::panic::catch_unwind(|| {
            generate_rust(&program);
        });

        assert!(result.is_err(), "Expected panic when passing arguments to Memoize closure");
    }
}
