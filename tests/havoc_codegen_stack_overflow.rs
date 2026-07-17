#![allow(missing_docs)]
use glossa::codegen::generate_rust;
use glossa::morphology::BinaryOp;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};

/// 👺 Havoc: Stack Overflow in Codegen
///
/// Sentry validation step prevents deep nesting.
#[test]
#[should_panic(expected = "Recursion limit exceeded in code generation")]
fn havoc_codegen_stack_overflow() {
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
    let program = Box::leak(Box::new(AnalyzedProgram {
        statements: vec![stmt],
        scope,
    }));

    // 💥 DETONATE
    let _ = generate_rust(program);
}
