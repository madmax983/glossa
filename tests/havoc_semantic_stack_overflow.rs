#![allow(missing_docs)]
use glossa::morphology::BinaryOp;
use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};

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
