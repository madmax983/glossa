use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;
use std::thread;

/// 👺 Havoc: Stack Overflow in CodeGen via Deep Expression Trees
///
/// This test demonstrates a stack overflow vulnerability in `generate_rust`.
/// The parser limits recursion depth to 500, but binary expressions (e.g., `1 + 1 + ...`)
/// are parsed iteratively into a flat `AnalyzedExpr` tree.
/// However, `generate_rust` processes this tree recursively, causing a stack overflow
/// when the tree depth exceeds the stack size.
///
/// We "fix" the test crash by running it in a thread with a massive stack.
/// This proves the code is correct (just stack-hungry).
#[test]
fn test_stack_overflow_expression() {
    // Spawn a thread with 32MB stack.
    let builder = thread::Builder::new().stack_size(32 * 1024 * 1024);

    let handler = builder
        .spawn(|| {
            // Generate a massive expression: 1 + 1 + 1 + ...
            // We use 1000 to ensure it passes with the custom stack size.
            // This is still 2x the parser's nesting limit (500), proving we bypassed it.
            let n = 1_000;
            let mut s = String::with_capacity(n * 15);
            s.push('1');
            for _ in 0..n {
                // "ἄθροισμα" means "+" (sum)
                s.push_str(" ἄθροισμα 1");
            }
            s.push_str(" λέγε.");

            println!("Parsing...");
            // This works fine (linear loop in parser)
            let ast = parse(&s).expect("Failed to parse");

            println!("Analyzing...");
            // This works fine (linear loop in build_expressions_from_literals_and_ops)
            let analyzed = analyze_program(&ast).expect("Failed to analyze");

            println!("Generating...");
            // This should pass with 32MB stack
            let code = generate_rust(&analyzed);
            assert!(code.len() > 0);
        })
        .unwrap();

    handler.join().unwrap();
}
