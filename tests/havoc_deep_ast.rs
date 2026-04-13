#![allow(missing_docs)]

use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
use std::env;
use std::process::Command;

#[test]
fn test_analyzed_expr_clone_overflow() {
    // If we are in the child process, execute the crashing code
    if env::var("HAVOC_TRIGGER_CRASH_CLONE").is_ok() {
        let mut expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: GlossaType::Boolean,
        };
        for _ in 0..20000 {
            expr = AnalyzedExpr {
                expr: AnalyzedExprKind::PropertyAccess {
                    owner: Box::new(expr),
                    property: String::from("prop").into(),
                },
                glossa_type: GlossaType::Boolean,
            };
        }
        let _ = expr.clone();
        return;
    }

    // Otherwise, we are in the test runner. Spawn the child process.
    let exe = env::current_exe().unwrap();
    let status = Command::new(exe)
        .arg("test_analyzed_expr_clone_overflow")
        .arg("--exact")
        .arg("--test-threads=1")
        .env("HAVOC_TRIGGER_CRASH_CLONE", "1")
        .status()
        .unwrap();

    // The process should crash/abort (non-zero exit code)
    assert!(!status.success(), "Process did not crash as expected!");
}

#[test]
fn test_analyzed_expr_drop_overflow() {
    // If we are in the child process, execute the crashing code
    if env::var("HAVOC_TRIGGER_CRASH_DROP").is_ok() {
        let mut expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: GlossaType::Boolean,
        };
        for _ in 0..20000 {
            expr = AnalyzedExpr {
                expr: AnalyzedExprKind::PropertyAccess {
                    owner: Box::new(expr),
                    property: String::from("prop").into(),
                },
                glossa_type: GlossaType::Boolean,
            };
        }
        return; // Drop happens here
    }

    // Otherwise, we are in the test runner. Spawn the child process.
    let exe = env::current_exe().unwrap();
    let status = Command::new(exe)
        .arg("test_analyzed_expr_drop_overflow")
        .arg("--exact")
        .arg("--test-threads=1")
        .env("HAVOC_TRIGGER_CRASH_DROP", "1")
        .status()
        .unwrap();

    // The process should crash/abort (non-zero exit code)
    assert!(!status.success(), "Process did not crash as expected!");
}
