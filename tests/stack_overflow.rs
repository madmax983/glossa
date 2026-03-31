use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
use std::thread;

const THREAD_STACK_SIZE: usize = 2 * 1024 * 1024; // 2MB

#[test]
#[ignore = "Demonstrates stack overflow panic during cloning a deeply nested AnalyzedExpr"]
fn test_analyzed_expr_clone_overflow() {
    let handle = thread::Builder::new()
        .name("test_clone_overflow".to_string())
        .stack_size(THREAD_STACK_SIZE)
        .spawn(|| {
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
        })
        .unwrap();

    let res = handle.join();
    assert!(
        res.is_err(),
        "Thread should have panicked due to stack overflow"
    );
}

#[test]
#[ignore = "Demonstrates stack overflow panic during dropping a deeply nested AnalyzedExpr"]
fn test_analyzed_expr_drop_overflow() {
    let handle = thread::Builder::new()
        .name("test_drop_overflow".to_string())
        .stack_size(THREAD_STACK_SIZE)
        .spawn(|| {
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
        })
        .unwrap();

    let res = handle.join();
    assert!(
        res.is_err(),
        "Thread should have panicked due to stack overflow"
    );
}
