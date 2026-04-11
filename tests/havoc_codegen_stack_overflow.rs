use glossa::codegen::generate_rust;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};

#[test]
#[ignore = "Demonstrates a SIGABRT stack overflow when directly generating code for deeply nested ASTs"]
fn test_codegen_deep_ast_overflow() {
    let depth = 50_000;

    // 👺 Havoc: Direct Stack Overflow in Codegen
    // We construct a deeply nested Analyzed AST manually, simulating a scenario where
    // either the parser/analyzer limits are bypassed, or an internal macro expands
    // into an unboundedly deep AST.
    //
    // The `generate_expr` function in `src/codegen.rs` does not use `stacker::maybe_grow`.
    // It relies entirely on standard Rust recursive function calls.
    // At a depth of 50,000, this will immediately blow the thread's stack limit
    // and crash the compiler backend with a fatal SIGABRT.

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
}
