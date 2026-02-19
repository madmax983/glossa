
use glossa::codegen::generate_rust;
use glossa::semantic::{AnalyzedProgram, AnalyzedStatement, AnalyzedExpr, AnalyzedExprKind, GlossaType, Scope};
use glossa::morphology::lexicon::BinaryOp;

#[test]
#[should_panic(expected = "Recursion limit exceeded")]
fn test_codegen_recursion_limit() {
    let depth = 500; // > MAX_RECURSION_DEPTH (200), but < Stack Limit (safe for Drop)

    let mut current_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(1),
        glossa_type: GlossaType::Number,
    };

    for _ in 0..depth {
        let right_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        };

        current_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BinOp {
                left: Box::new(current_expr),
                op: BinaryOp::Add,
                right: Box::new(right_expr),
            },
            glossa_type: GlossaType::Number,
        };
    }

    let program = AnalyzedProgram {
        statements: vec![
            AnalyzedStatement::Expression(vec![current_expr])
        ],
        scope: Scope::new(),
    };

    let _code = generate_rust(&program);
}
