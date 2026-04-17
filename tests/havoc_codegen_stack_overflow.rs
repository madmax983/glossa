#![allow(missing_docs)]

use glossa::codegen::generate_rust;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};
use std::env;
use std::process::Command;

#[test]
fn test_codegen_deep_ast_overflow() {
    // If we are in the child process, execute the crashing code
    if env::var("HAVOC_TRIGGER_CRASH_CODEGEN").is_ok() {
        let depth = 50_000;

        let mut expr = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        };

        for _ in 0..depth {
            expr = AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(expr),
                    method: "clone".into(),
                    args: vec![],
                },
                glossa_type: GlossaType::Number,
            };
        }

        let stmt = AnalyzedStatement::Expression(vec![expr]);

        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope: Scope::new(),
        };

        // 💥 DETONATE
        let _rust = generate_rust(&program);
        return;
    }

    // Otherwise, we are in the test runner. Spawn the child process.
    let exe = env::current_exe().unwrap();
    let status = Command::new(exe)
        .arg("test_codegen_deep_ast_overflow")
        .arg("--exact")
        .arg("--test-threads=1")
        .env("HAVOC_TRIGGER_CRASH_CODEGEN", "1")
        .status()
        .unwrap();

    // The process should crash/abort (non-zero exit code)
    assert!(!status.success(), "Process did not crash as expected!");
}
