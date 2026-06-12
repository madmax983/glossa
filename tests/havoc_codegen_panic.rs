#![allow(missing_docs)]
use glossa::codegen::generate_rust;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};
use std::env;
use std::process::Command;

/// 👺 Havoc: Stack Overflow in Codegen (Direct)
///
/// Warden previously mitigated parsing and execution limits, but
/// codegen (`generate_rust`) is fully recursive and lacks stacker protections.
/// A massive AST passed directly into codegen causes an immediate stack overflow.
#[test]
fn havoc_codegen_stack_overflow_direct() {
    if env::var("HAVOC_DETONATE_CODEGEN_PANIC").is_ok() {
        let depth = 50_000;
        let mut expr = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        };
        for _ in 0..depth {
            expr = AnalyzedExpr {
                expr: AnalyzedExprKind::BinOp {
                    left: Box::new(expr),
                    op: glossa::morphology::BinaryOp::Add,
                    right: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                },
                glossa_type: GlossaType::Number,
            };
        }

        let stmt = AnalyzedStatement::Expression(vec![expr]);
        let scope = Scope::new();
        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope,
        };

        let _ = generate_rust(&program);
        std::process::exit(0);
    }

    let exe = env::current_exe().expect("Failed to get current executable");

    let status = Command::new(exe)
        .env("HAVOC_DETONATE_CODEGEN_PANIC", "1")
        .arg("--nocapture")
        .arg("havoc_codegen_stack_overflow_direct")
        .status()
        .expect("Failed to spawn subprocess");

    assert!(
        !status.success(),
        "Subprocess should have crashed due to stack overflow!"
    );
}
