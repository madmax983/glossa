#[cfg(feature = "nova")]
use glossa::AnalyzedProgram;
#[cfg(feature = "nova")]
use glossa::morphology::UnaryOp;
#[cfg(feature = "nova")]
use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope};
#[cfg(feature = "nova")]
use glossa::tools::interpreter::Interpreter;

#[cfg(feature = "nova")]
#[test]
#[ignore = "Demonstrates stack overflow in interpreter"]
fn test_interpreter_stack_overflow() {
    let mut expr = AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(1),
        glossa_type: GlossaType::Number,
    };

    // Construct a deeply nested expression
    for _ in 0..10_000 {
        expr = AnalyzedExpr {
            expr: AnalyzedExprKind::UnaryOp {
                op: UnaryOp::Neg,
                operand: Box::new(expr),
            },
            glossa_type: GlossaType::Number,
        };
    }

    let stmt = AnalyzedStatement::Expression(vec![expr]);
    let program = AnalyzedProgram {
        statements: vec![stmt],
        scope: Scope::new(),
    };

    let mut interpreter = Interpreter::new();
    let _ = interpreter.run(&program);
}
