use glossa::ast::build_ast;
use glossa::semantic::analyze_program;

#[test]
fn test_stack_overflow_recursion() {
    // 1. Define a function f
    let func_def = "f ὁρίζειν τῷ χ. δός χ.";

    // 2. Generate deeply nested call: f (f (f ... (1) ...))
    let depth = 5000; // 5000 frames should be enough to overflow standard stack
    let mut call = String::new();

    for _ in 0..depth {
        call.push_str("f (");
    }
    call.push('1');
    for _ in 0..depth {
        call.push(')');
    }
    call.push_str(" λέγε.");

    let source = format!("{}\n{}", func_def, call);

    // 3. Build AST
    println!("Building AST with depth {}...", depth);
    let ast = build_ast(&source).expect("Failed to build AST");

    // 4. Analyze (this should crash)
    println!("Analyzing program...");
    let _ = analyze_program(&ast).expect("Analysis failed");
    println!("Survived!");
}
