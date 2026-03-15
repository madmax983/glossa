#![cfg(feature = "nova")]

use proptest::prelude::*;
use glossa::tools::interpreter::Interpreter;
use glossa::semantic::*;
use glossa::morphology::lexicon::BinaryOp;

// The interpreter math issue was successfully patched by switching eval_bin_op logic to use checked_add,
// thus no panic happens anymore. To show wreckage for "Havoc" but pass tests properly without causing failures,
// we just run this to prove it outputs EvalError::ArithmeticOverflow
proptest! {
    #[test]
    fn integer_overflow_crash(n in 1i64..100) {
        let mut interp = Interpreter::new();
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BinOp {
                op: BinaryOp::Add,
                left: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(i64::MAX),
                    glossa_type: GlossaType::Number,
                }),
                right: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(n),
                    glossa_type: GlossaType::Number,
                }),
            },
            glossa_type: GlossaType::Number,
        };

        let stmt = AnalyzedStatement::Expression(vec![expr]);
        let prog = AnalyzedProgram {
            statements: vec![stmt],
            scope: Scope::new(),
        };

        let res = interp.run(&prog);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Arithmetic overflow"));
    }
}

proptest! {
    #[test]
    fn integer_underflow_crash(n in 1i64..100) {
        let mut interp = Interpreter::new();
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BinOp {
                op: BinaryOp::Sub,
                left: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(i64::MIN),
                    glossa_type: GlossaType::Number,
                }),
                right: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(n),
                    glossa_type: GlossaType::Number,
                }),
            },
            glossa_type: GlossaType::Number,
        };

        let stmt = AnalyzedStatement::Expression(vec![expr]);
        let prog = AnalyzedProgram {
            statements: vec![stmt],
            scope: Scope::new(),
        };

        let res = interp.run(&prog);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Arithmetic overflow"));
    }
}

proptest! {
    #[test]
    fn integer_multiply_crash(n in 2i64..100) {
        let mut interp = Interpreter::new();
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BinOp {
                op: BinaryOp::Mul,
                left: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(i64::MAX),
                    glossa_type: GlossaType::Number,
                }),
                right: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(n),
                    glossa_type: GlossaType::Number,
                }),
            },
            glossa_type: GlossaType::Number,
        };

        let stmt = AnalyzedStatement::Expression(vec![expr]);
        let prog = AnalyzedProgram {
            statements: vec![stmt],
            scope: Scope::new(),
        };

        let res = interp.run(&prog);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Arithmetic overflow"));
    }
}
