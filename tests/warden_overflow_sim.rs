#![allow(missing_docs)]
#![cfg(feature = "nova")]
use glossa::morphology::{BinaryOp, UnaryOp};
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};
use glossa::tools::Interpreter;

#[test]
fn test_warden_overflow_sim_defense() {
    let mut interpreter = Interpreter::new();

    let cases = vec![
        // Add max + 1
        (BinaryOp::Add, i64::MAX, 1),
        // Sub min - 1
        (BinaryOp::Sub, i64::MIN, 1),
        // Mul max * 2
        (BinaryOp::Mul, i64::MAX, 2),
        // Div min / -1
        (BinaryOp::Div, i64::MIN, -1),
    ];

    for (op, l, r) in cases {
        let number_expr_l = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(l),
            glossa_type: GlossaType::Number,
        };
        let number_expr_r = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(r),
            glossa_type: GlossaType::Number,
        };
        let bin_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BinOp {
                left: Box::new(number_expr_l),
                op,
                right: Box::new(number_expr_r),
            },
            glossa_type: GlossaType::Number,
        };

        let stmt = AnalyzedStatement::Expression(vec![bin_expr]);

        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope: Scope::new(),
        };

        let result = interpreter.run(&program);
        assert!(
            result.is_err(),
            "Interpreter should return an error on overflow instead of panicking"
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("overflow") || err.to_string().contains("ὑπερχείλισις"),
            "Expected overflow error, got: {}",
            err
        );
    }

    // Test Negate i64::MIN
    let number_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(i64::MIN),
        glossa_type: GlossaType::Number,
    };
    let neg_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::UnaryOp {
            op: UnaryOp::Neg,
            operand: Box::new(number_expr),
        },
        glossa_type: GlossaType::Number,
    };

    let stmt2 = AnalyzedStatement::Expression(vec![neg_expr]);

    let program2 = AnalyzedProgram {
        statements: vec![stmt2],
        scope: Scope::new(),
    };

    let result2 = interpreter.run(&program2);
    assert!(
        result2.is_err(),
        "Interpreter should return an error on overflow instead of panicking for unary neg"
    );
    let err = result2.unwrap_err();
    assert!(
        err.to_string().contains("overflow") || err.to_string().contains("ὑπερχείλισις"),
        "Expected overflow error, got: {}",
        err
    );
}
