#![allow(missing_docs)]
use glossa::codegen::generate_rust;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, CaptureMode, GlossaType,
    Scope,
};

#[test]
fn test_security_memoize_repro() {
    // This test manually constructs a semantic model that uses CaptureMode::Memoize
    // with a closure that takes arguments.
    //
    // Current behavior: Generates a standard closure without internal caching mechanisms
    // Desired behavior: Should safely generate a normal closure instead of panicking.

    let body = AnalyzedExpr {
        expr: AnalyzedExprKind::Variable("x".into()),
        glossa_type: GlossaType::Number,
    };

    let lambda = AnalyzedExpr {
        expr: AnalyzedExprKind::Lambda {
            params: vec!["x".into()],
            body: Box::new(body),
            capture_mode: CaptureMode::Memoize,
        },
        glossa_type: GlossaType::Function {
            params: vec![GlossaType::Number],
            returns: Box::new(GlossaType::Number),
        },
    };

    let stmt = AnalyzedStatement::Expression(vec![lambda]);

    let program = AnalyzedProgram {
        statements: vec![stmt],
        scope: Scope::new(),
    };

    let code = generate_rust(&program);

    println!("Generated code:\n{}", code);

    // Ensure that it does NOT contain the unsafe RefCell memoization block
    assert!(
        !code.contains("RefCell::new(None)"),
        "Should not generate memoized caching code for closures with arguments"
    );
    assert!(!code.contains("cache_ref.is_none()"));

    // Ensure it generated a standard closure that takes the arguments
    // Quote formats closures like `move | g_x |` with spaces.
    assert!(
        code.contains("move | g_x |"),
        "Should fallback to normal closure generation"
    );
}
