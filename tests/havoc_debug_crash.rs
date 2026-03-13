use glossa::ast::Expr;

/// 👺 Havoc: Stack Overflow in Derived Debug
///
/// Warden meticulously secured `Drop`, `Clone`, and `PartialEq`
/// using `stacker::maybe_grow`. But the derived `Debug` implementation
/// is left completely exposed.
///
/// If an error occurs during parsing or semantic analysis, the compiler
/// might print the AST using `{:?}`. A deeply nested tree will bypass
/// all protections and crash the process with a Stack Overflow.
#[test]
fn havoc_crash_debug_stack_overflow() {
    let depth = 50_000;

    // Construct a deeply nested expression
    let mut deep_expr = Expr::NumberLiteral(1);
    for _ in 0..depth {
        deep_expr = Expr::Phrase(vec![deep_expr]);
    }

    // 💥 DETONATE
    // The derived `Debug` implementation recurses into `Expr::Phrase`
    // without `stacker::maybe_grow` or explicit loop.
    // This will immediately exhaust the thread stack and crash the runner.
    println!("Formatting deep expression (depth {})...", depth);
    let _s = format!("{:?}", deep_expr);

    // This line will never be reached!
    println!("Survived? Impossible.");
}
