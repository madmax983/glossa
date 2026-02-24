use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_nested_while_stack_overflow() {
    // Generate a massive nested While loop: ἕως 1, ἕως 1, ... 1.
    // Each "ἕως 1," adds a level of recursion in parse_while_loop -> analyze_statement.
    // 5,000 iterations -> Depth 5,000.
    // Parser sees a flat list of clauses (no recursion limit).
    // Semantic analysis recurses.

    // We need enough depth to overflow the stack.
    // Default stack is usually 2MB.
    // 5000 frames * ~1KB/frame = ~5MB -> Should crash.
    let n = 5_000;
    let mut s = String::with_capacity(n * 20);

    // Build the string: "ἕως 1, ἕως 1, ... 1."
    for _ in 0..n {
        s.push_str("ἕως 1, ");
    }
    s.push_str("1.");

    println!("Parsing {} nested while loops...", n);
    // Parsing should succeed (flat structure of clauses separated by commas)
    let ast = parse(&s).expect("Failed to parse");
    println!(
        "Parsed {} statements. First statement has {} clauses.",
        ast.statements.len(),
        ast.statements[0].clauses().len()
    );

    println!("Analyzing...");
    // This should Stack Overflow (crash).
    // If it doesn't, we failed (or stack is huge).
    let result = analyze_program(&ast);

    // If we reach here, we check if it failed gracefully
    match result {
        Ok(_) => panic!("Analysis succeeded unexpectedly! Should have crashed or returned LimitExceeded."),
        Err(e) => {
             // If we fixed it, it should be LimitExceeded.
             // If we haven't, it crashed before here.
             match e {
                glossa::errors::GlossaError::LimitExceeded { resource, max } => {
                    println!("Caught recursion limit: {} (max {})", resource, max);
                }
                _ => panic!("Expected LimitExceeded, got {:?}", e),
            }
        }
    }
}
