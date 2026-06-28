#![allow(missing_docs)]
use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
use glossa::morphology::BinaryOp;
use proptest::prelude::*;
use std::env;
use std::process::Command;

#[test]
fn havoc_proptest_deep_ast_stack_overflow() {
    if env::var("HAVOC_DETONATE_PROPTEST").is_ok() {
        let depth = 50_000;
        let mut expr = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        };
        for _ in 0..depth {
            expr = AnalyzedExpr {
                expr: AnalyzedExprKind::BinOp {
                    left: Box::new(expr),
                    op: BinaryOp::Add,
                    right: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                },
                glossa_type: GlossaType::Number,
            };
        }
        // Implicit drop(expr) happens here.
        // If depth is large enough, the recursive Drop implementation will stack overflow,
        // proving the system's fragility.
        drop(expr);
        std::process::exit(0);
    }

    let exe = env::current_exe().unwrap();
    let status = Command::new(exe)
        .env("HAVOC_DETONATE_PROPTEST", "1")
        .arg("--nocapture")
        .arg("havoc_proptest_deep_ast_stack_overflow")
        .status()
        .unwrap();

    assert!(!status.success(), "Proptest stack overflow trigger failed!");
}
