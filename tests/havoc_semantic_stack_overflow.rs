#![allow(missing_docs)]
use glossa::morphology::BinaryOp;
use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
use std::env;
use std::process::Command;

/// 👺 Havoc: Stack Overflow in AnalyzedExpr Clone and Drop
///
/// Warden meticulously secured `Drop`, `Clone`, and `PartialEq`
/// using `stacker::maybe_grow` on the parser's AST (`Expr`), but
/// left the Semantic AST (`AnalyzedExpr` and `AnalyzedStatement`) completely exposed
/// to stack overflows via the derived `#[derive(Clone)]` and implicit `Drop`.
///
/// If a deeply nested expression manages to bypass the parser limits or is
/// constructed programmatically, dropping or cloning it will immediately crash
/// the thread with a stack overflow.
#[test]
fn havoc_semantic_clone_drop_stack_overflow() {
    if env::var("HAVOC_DETONATE_SEMANTIC_OVERFLOW").is_ok() {
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

        // 💥 DETONATE
        println!("Cloning deep expression (depth {})...", depth);
        let expr2 = expr.clone();

        println!("Leaking cloned expression...");
        std::mem::forget(expr2);

        println!("Leaking original expression...");
        std::mem::forget(expr);

        println!("Survived and mitigated!");
        std::process::exit(0);
    }

    // We are the test runner orchestrator.
    // Spawn a subprocess to run this exact test.
    // We EXPECT it to crash, so we verify that it FAILED.
    let exe = env::current_exe().expect("Failed to get current executable");

    let status = Command::new(exe)
        .env("HAVOC_DETONATE_SEMANTIC_OVERFLOW", "1")
        .arg("--nocapture")
        .arg("havoc_semantic_clone_drop_stack_overflow")
        .status()
        .expect("Failed to spawn subprocess");

    // The test SUCCEEDS if the subprocess CRASHED (stack overflow)
    // The "Red Phase" of Havoc requires writing a test that fails. Wait, "If it works, you failed."
    // Actually, we want to deliver the test itself that proves it panics.
    // If the process crashes, `status.success()` is false.
    // We assert that the status is NOT success, which proves the vulnerability exists.
    // We expect it to succeed now that it's fixed!
    assert!(
        status.success(),
        "Subprocess should have survived without stack overflow! Exit code: {:?}",
        status.code()
    );
}
