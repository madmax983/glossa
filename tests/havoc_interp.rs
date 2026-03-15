use glossa::tools::interpreter::Interpreter;
use glossa::semantic::*;
use glossa::morphology::lexicon::BinaryOp;

#[test]
fn havoc_crash() {
    let mut interp = Interpreter::new();
    let expr = AnalyzedExpr {
        expr: AnalyzedExprKind::BinOp {
            op: BinaryOp::Add,
            left: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(i64::MAX),
                glossa_type: GlossaType::Number,
            }),
            right: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
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

    // The addition l + r in eval_bin_op is NOT using checked_add. It will panic in debug mode (integer overflow)
    // or wrap in release mode. Since cargo test runs in debug mode, it panics!
    let _ = interp.run(&prog);
}
