use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_elif_chain_stack_overflow() {
    // Generate a massive elif chain: εἰ 1, 1, εἰ 1, 1, ...
    // Each "εἰ 1, 1," adds a level of recursion in parse_conditional.
    // 2,000 iterations -> Depth 2,000.
    // Parser sees a flat list of clauses (no recursion limit).
    // Semantic analysis recurses.

    let n = 2_000;
    let mut s = String::with_capacity(n * 20);

    // First if
    s.push_str("εἰ 1, 1");

    // Use middle dot (·) to chain else-if, because parse_conditional expects
    // the else-if to be the second expression of the then-clause.
    // Logic: if then_clause.expressions.len() > 1 ...
    for _ in 0..n {
        // Else if chained with middle dot
        s.push_str(" · εἰ 1, 1");
    }
    s.push('.');

    println!("Parsing {} elif clauses...", n);
    // Parsing should succeed (flat structure)
    let ast = parse(&s).expect("Failed to parse");
    println!(
        "Parsed {} statements. First statement has {} clauses.",
        ast.statements.len(),
        ast.statements[0].clauses().len()
    );

    println!("Analyzing...");
    // This should NOT Stack Overflow anymore. It should return LimitExceeded.
    let result = analyze_program(&ast);

    match result {
        Ok(_) => panic!("Expected LimitExceeded error, but analysis succeeded!"),
        Err(e) => {
            println!("Got expected error: {:?}", e);
            match e {
                glossa::errors::GlossaError::LimitExceeded { resource, max } => {
                    assert!(
                        resource == "Control flow depth" || resource == "statement analysis depth"
                    );
                    assert!(max == 100 || max == 50);
                }
                _ => panic!("Expected LimitExceeded, got {:?}", e),
            }
        }
    }
}
