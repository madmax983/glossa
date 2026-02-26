use glossa::ast::Expr;

#[test]
fn test_deep_recursion_drop_and_clone() {
    // We run this in a separate thread with a larger stack to verify it DOES NOT crash
    // if the fix is applied.
    // However, to reproduce the crash initially, we rely on the default stack size.
    // But since `cargo test` runs tests in threads with 2MB stack (usually),
    // 20,000 recursive calls will definitely overflow.

    // Depth: 100,000
    let depth = 100_000;

    // Construct deep expression: Phrase([Phrase([Phrase(...)])])
    let mut deep_expr = Expr::NumberLiteral(1);
    for _ in 0..depth {
        deep_expr = Expr::Phrase(vec![deep_expr]);
    }

    // 1. Test Clone (should stack overflow if recursive)
    println!("Cloning deep expression (depth {})...", depth);
    let cloned = deep_expr.clone();

    // 2. Test Drop (should stack overflow if recursive)
    println!("Dropping cloned expression...");
    drop(cloned);

    println!("Dropping original expression...");
    drop(deep_expr);

    println!("Success!");
}
