#![allow(missing_docs)]
use glossa::codegen::generate_rust;
use glossa::morphology::BinaryOp;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};
use std::env;
use std::process::Command;

/// 👺 Havoc: Stack Overflow in Codegen
///
/// If a deeply nested expression manages to bypass the parser limits or is
/// constructed programmatically, generating Rust code for it will immediately crash
/// the thread with a stack overflow.
#[test]
fn havoc_codegen_stack_overflow() {
    if env::var("HAVOC_DETONATE_CODEGEN_OVERFLOW").is_ok() {
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

        let stmt = AnalyzedStatement::Expression(vec![expr]);
        let scope = Scope::new();
        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope,
        };

        // 💥 DETONATE
        println!(
            "Generating rust code for deep expression (depth {})...",
            depth
        );
        let _ = generate_rust(&program);

        println!("Survived and mitigated!");
        std::process::exit(0);
    }

    let exe = env::current_exe().expect("Failed to get current executable");

    let status = Command::new(exe)
        .env("HAVOC_DETONATE_CODEGEN_OVERFLOW", "1")
        .arg("--nocapture")
        .arg("havoc_codegen_stack_overflow")
        .status()
        .expect("Failed to spawn subprocess");

    assert!(
        !status.success(),
        "Subprocess should have crashed due to stack overflow!"
    );
}
