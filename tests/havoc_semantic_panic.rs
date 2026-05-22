#![allow(missing_docs)]
use glossa::morphology::BinaryOp;
use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};

/// 👺 Havoc: True Stack Overflow in AnalyzedExpr Clone and Drop
///
/// This test explicitly bypasses the test suite's subprocess mitigation
/// by detonating directly within the main thread, proving that the semantic
/// AST (`AnalyzedExpr` and `AnalyzedStatement`) is exposed to stack overflows
/// via the derived `#[derive(Clone)]` and implicit `Drop`.
#[test]
fn havoc_semantic_panic() {
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

    println!("Dropping cloned expression...");
    drop(expr2);

    println!("Dropping original expression...");
    drop(expr);
}
