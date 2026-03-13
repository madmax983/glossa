use glossa::ast::Expr;

/// 👺 Havoc: Stack Overflow in Derived Clone
///
/// While `Drop` for `Expr` is protected by `stacker::maybe_grow`,
/// the derived `Clone` and `PartialEq` implementations are NOT.
///
/// A deeply nested AST node will bypass the parser's depth limit
/// if created programmatically (or through a future macro/FFI expansion).
/// Calling `.clone()` on this node causes an uncontrolled stack overflow,
/// crashing the thread/process instantly.
///
/// Running this test will terminate the runner with Exit Code 139 (SIGSEGV).
#[test]
fn havoc_crash_clone_stack_overflow() {
    let depth = 1_000_000;

    // Construct a deeply nested expression
    let mut deep_expr = Expr::NumberLiteral(1);
    for _ in 0..depth {
        deep_expr = Expr::Phrase(vec![deep_expr]);
    }

    // 💥 DETONATE
    // The derived `Clone` implementation recurses into `Expr::Phrase`
    // without `stacker::maybe_grow` or explicit loop.
    // This will immediately exhaust the thread stack and crash the runner.
    println!("Cloning deep expression (depth {})...", depth);
    let _cloned = deep_expr.clone();

    // This line will never be reached!
    println!("Survived? Impossible.");
}
