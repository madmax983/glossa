use glossa::parser::parse;
use glossa::semantic::analyze_program;

/// 👺 Havoc: Resource Limits Prevention
///
/// This test verifies that massive expressions trigger `LimitExceeded`
/// instead of proceeding to potentially stack-overflowing codegen.
///
/// Previously, this test used a massive stack to prove that deep recursion works.
/// Now, with strict resource limits, it proves that such recursion is blocked early.
#[test]
fn test_stack_overflow_expression_prevented() {
    // Generate a massive expression: 1 + 1 + 1 + ...
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
    // This should FAIL with LimitExceeded because we have too many literals/operators
    let result = analyze_program(&ast);

    match result {
        Ok(_) => panic!(
            "Expected LimitExceeded error, but analysis succeeded! The resource limits are not working."
        ),
        Err(e) => {
            let msg = e.to_string();
            println!("Got expected error: {}", msg);
            // Check for localized message or debug variant name
            assert!(
                msg.contains("LimitExceeded") || msg.contains("Ὑπέρβασις ὁρίων"),
                "Expected LimitExceeded error, got: {}",
                msg
            );
        }
    }
}
