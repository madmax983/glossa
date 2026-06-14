#![allow(missing_docs)]
use glossa::codegen::generate_rust;
use glossa::morphology::BinaryOp;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};

/// 👺 Havoc: Stack Overflow in Codegen (Direct)
///
/// If a deeply nested expression manages to bypass the parser limits or is
/// constructed programmatically, generating Rust code for it will immediately crash
/// the thread with a stack overflow.
#[test]
fn havoc_codegen_stack_overflow_direct() {
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
}
