use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

/// 👺 Havoc: Stack Overflow in CodeGen via Deep Expression Trees
///
/// This test demonstrates a stack overflow vulnerability in `generate_rust`.
/// The parser limits recursion depth to 500, but binary expressions (e.g., `1 + 1 + ...`)
/// are parsed iteratively into a flat `AnalyzedExpr` tree.
/// However, `generate_rust` processes this tree recursively, causing a stack overflow
/// when the tree depth exceeds the stack size (e.g., 20,000 additions).
///
/// To reproduce the crash, run:
/// `cargo test --test havoc_overflow -- --ignored`
#[test]
#[ignore = "Demonstrates a stack overflow crash. Run explicitly to see the wreckage."]
fn test_stack_overflow_expression() {
    // Generate a massive expression: 1 + 1 + 1 + ...
    // Depth 20,000 creates a recursion depth of ~20,000 in generate_expr, blowing the stack.
    let n = 20_000;
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
    // This crashes with fatal runtime error: stack overflow
    let _code = generate_rust(&analyzed);
}
