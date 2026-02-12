use glossa::codegen::generate_rust;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, CaptureMode, GlossaType,
    Scope,
};

#[test]
#[should_panic(expected = "Memoization is only supported for 0-argument closures")]
fn test_security_memoize_repro() {
    // This test manually constructs a semantic model that uses CaptureMode::Memoize
    // with a closure that takes arguments.
    //
    // Current behavior: Generates code that caches the result ignoring arguments.
    // Desired behavior: Should panic during compilation because this is unsafe.

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

    // If we reach here, compilation succeeded (which means the vulnerability exists).
    // We check if the generated code ignores the argument.
    // The pattern for memoization is:
    // if cache_ref.is_none() { *cache_ref = Some(body); }
    // Note that 'x' is not used in the key.

    println!("Generated code:\n{}", code);

    if code.contains("RefCell::new(None)") && code.contains("if cache_ref.is_none()") {
        // Confirm it takes arguments
        if code.contains("|g_x|") || code.contains("|x|") {
            // It compiled successfully but generated unsafe code.
            // We want this test to fail initially (by NOT panicking),
            // but since we want to demonstrate the fix, we set #[should_panic].
            //
            // Wait, if I want to demonstrate the vulnerability, I should assert the bad code exists.
            // But the plan says "Update to expect a panic".
            // So I will set #[should_panic] now, anticipating the fix.
            // But BEFORE the fix, this test will FAIL (it won't panic, it will print code).
            // That's fine.
        }
    }

    // To ensure the test fails BEFORE the fix (proving the vulnerability exists and is silent),
    // and PASSES after the fix (proving the panic is triggered),
    // I should assert that we are NOT panicking here?
    // No, I want to enforce the panic.
    // So #[should_panic] is correct for the final state.
    // Before the fix, `generate_rust` returns string, so no panic.
    // So the test (with should_panic) will FAIL.
    // After the fix, `generate_rust` panics. The test (with should_panic) will PASS.
}
