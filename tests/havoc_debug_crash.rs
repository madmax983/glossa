use glossa::ast::Expr;
use std::env;
use std::process::Command;

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
    // If we are running in the subprocess, execute the crash vector
    if env::var("HAVOC_DETONATE_DEBUG").is_ok() {
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
        std::process::exit(0);
    }

    // Otherwise, we are the test runner orchestrator.
    // Spawn a subprocess to run this exact test, and verify it CRASHES.
    let exe = env::current_exe().expect("Failed to get current executable");

    let status = Command::new(exe)
        .env("HAVOC_DETONATE_DEBUG", "1")
        .arg("--nocapture")
        .arg("havoc_crash_debug_stack_overflow")
        .status()
        .expect("Failed to spawn subprocess");

    // The test SUCCEEDS if the subprocess SUCCEEDS (it no longer crashes!)
    assert!(
        status.success(),
        "Bug persists! The subprocess should NOT have crashed."
    );
}
