use glossa::errors::GlossaError;
use glossa::errors::assembly::AssemblyError;
use glossa::parser::parse;
use glossa::semantic::analyze_program;
use std::thread;

/// 👺 Havoc: Stack Overflow in CodeGen via Deep Expression Trees
///
/// This test originally demonstrated a stack overflow vulnerability in `generate_rust`.
/// However, with the new resource limits in `Assembler`, this malicious input
/// is now rejected early during the analysis phase.
///
/// We verify that the system gracefully handles this DoS attempt by returning
/// a `LimitExceeded` error instead of crashing or proceeding to code generation.
#[test]
fn test_stack_overflow_expression_prevented() {
    // Spawn a thread with normal stack (limits should kick in first).
    let builder = thread::Builder::new();

    let handler = builder
        .spawn(|| {
            // Generate a massive expression: 1 + 1 + 1 + ...
            // We use 2000 to ensure it exceeds the assembler limits (MAX_LITERALS=1024).
            let n = 2_000;
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
            // This should now FAIL with LimitExceeded during analysis.
            let result = analyze_program(&ast);

            match result {
                Ok(_) => panic!("Expected LimitExceeded error, but analysis succeeded! Resource limits may be broken."),
                Err(GlossaError::AssemblyError(AssemblyError::LimitExceeded { .. })) => {
                    println!("Successfully prevented stack overflow via resource limits.");
                },
                Err(e) => panic!("Expected LimitExceeded error, but got: {:?}", e),
            }
        })
        .unwrap();

    handler.join().unwrap();
}
