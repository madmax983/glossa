#![allow(missing_docs)]
use glossa::codegen::generate_rust;
use glossa::morphology::BinaryOp;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};

/// 👺 Havoc: Stack Overflow in Codegen
///
/// If a deeply nested expression manages to bypass the parser limits or is
/// constructed programmatically, generating Rust code for it will immediately crash
/// the thread with a stack overflow.
#[test]
#[should_panic(expected = "Codegen depth limit exceeded")]
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
    let program = AnalyzedProgram {
        statements: vec![stmt],
        scope,
    };

    println!(
        "Generating rust code for deep expression (depth {})...",
        depth
    );

    // Prevent the AST from blowing the stack when it drops, by leaking it.
    // The test is verifying that codegen itself doesn't crash with a stack overflow,
    // but rather panics safely before running out of stack.
    let program_ref: &'static AnalyzedProgram = Box::leak(Box::new(program));

    // 💥 DETONATE
    let _ = generate_rust(program_ref);
}
