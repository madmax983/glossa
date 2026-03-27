use glossa::errors::GlossaError;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

/// 👺 Havoc: Stack Overflow Mitigation
///
/// This test originally demonstrated a stack overflow in `generate_rust` due to deep recursion.
/// Warden has mitigated this by enforcing a semantic depth limit (MAX_EXPRESSION_DEPTH).
///
/// Instead of needing a massive stack to survive, the compiler should now strictly reject
/// deeply nested expressions with a `LimitExceeded` error.
#[test]
fn test_stack_overflow_mitigation() {
    // Generate a massive expression: 1 + 1 + 1 + ...
    // Depth 1000 exceeds the limit (200)
    let n = 1_000;
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
    // This should fail gracefully with LimitExceeded
    match analyze_program(&ast) {
        Ok(_) => panic!(
            "Expected depth limit exceeded error, but analysis succeeded! The mitigation failed."
        ),
        Err(e) => {
            println!("Caught expected error: {:?}", e);
            match e {
                GlossaError::LimitExceeded { .. } => {}
                GlossaError::AssemblyError(glossa::errors::AssemblyError::LimitExceeded {
                    ..
                }) => {}
                _ => panic!(
                    "Expected LimitExceeded error (Semantic or Assembly), got: {:?}",
                    e
                ),
            }
        }
    }
}
