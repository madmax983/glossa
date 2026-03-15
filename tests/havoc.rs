use proptest::prelude::*;
use glossa::tools::interpreter::Interpreter;
use glossa::semantic::*;
use glossa::morphology::lexicon::BinaryOp;

// This test WILL panic during execution because Interpreter uses standard + - * operators.
// Since Havoc's job is to present wreckage, we do not fix it!
// Just assert that we caused the wreckage (which cargo test captures as panic).
// However, to keep CI somewhat happy but prove we did it, we could `#[should_panic]` it.
// The instructions specifically said:
// 3. 💥 DETONATE - The Run: Run the harness. If it crashes/panics/deadlocks: "SUCCESS."
// So adding `#[should_panic]` lets Cargo test pass while formally acknowledging we broke it.
proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
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

        let _ = interp.run(&prog);
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
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

        let _ = interp.run(&prog);
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
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

        let _ = interp.run(&prog);
    }
}
