use glossa::ast::{Expr, Word};
use std::thread;

#[test]
fn test_clone_stack_overflow() {
    let child = thread::Builder::new()
        .stack_size(1024 * 1024 * 10) // generous but still droppable
        .spawn(|| {
            let depth = 50000;
            let mut expr = Expr::Word(Word::new("root"));
            for _ in 0..depth {
                expr = Expr::PropertyAccess {
                    owner: Box::new(expr),
                    property: Box::new(Expr::Word(Word::new("prop"))),
                };
            }

            // Stacker should protect Clone according to ast.rs
            let expr2 = expr.clone();
            // Also test dropping it
            drop(expr2);
            drop(expr);
        })
        .unwrap();
    child.join().unwrap();
}

use glossa::morphology::BinaryOp;
use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
use std::env;
use std::process::Command;

/// 👺 Havoc: AnalyzedExpr Drop Stack Overflow
///
/// Ensures we can crash the process with a deep `AnalyzedExpr` drop.
#[test]
fn test_havoc_analyzedexpr_drop() {
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

        // 💥 Detonate
        println!("Dropping cloned expression...");
        drop(expr);

        println!("Survived and mitigated!");
        std::process::exit(0);
    }

    // Spawn a subprocess to run this exact test.
    let exe = env::current_exe().expect("Failed to get current executable");

    let status = Command::new(exe)
        .env("HAVOC_DETONATE_SEMANTIC_OVERFLOW", "1")
        .arg("--nocapture")
        .arg("test_havoc_analyzedexpr_drop")
        .status()
        .expect("Failed to spawn subprocess");

    assert!(
        !status.success(),
        "Subprocess should have crashed due to stack overflow!"
    );
}
