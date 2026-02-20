use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_stack_overflow_in_drop() {
    // Generate a massive expression: 1 + 1 + 1 + ...
    // Depth 100,000 should blow the stack during Drop
    let n = 1_000_000;
    let mut s = String::with_capacity(n * 15);
    s.push('1');
    for _ in 0..n {
        // "ἄθροισμα" means "+" (sum)
        s.push_str(" ἄθροισμα 1");
    }
    s.push_str(" λέγε.");

    println!("Parsing...");
    let ast = parse(&s).expect("Failed to parse");

    println!("Analyzing...");
    println!("Size of AnalyzedExpr: {}", std::mem::size_of::<glossa::semantic::AnalyzedExpr>());

    // This should fail with LimitExceeded, but then panic during Drop
    let result = analyze_program(&ast);

    match &result {
        Ok(_) => panic!("Analysis SUCCEEDED (unexpected)"),
        Err(e) => {
            println!("Analysis FAILED (expected): {:?}", e);
            match e {
                glossa::errors::GlossaError::AssemblyError(
                    glossa::semantic::AssemblyError::LimitExceeded { resource, max }
                ) => {
                    assert_eq!(resource, "Operators");
                    assert_eq!(*max, 256);
                }
                _ => panic!("Expected AssemblyError::LimitExceeded, got {:?}", e),
            }
        }
    }
    // Result is dropped here. Since analysis failed early (256 limit), the tree is shallow.
    // No stack overflow should occur.
}
