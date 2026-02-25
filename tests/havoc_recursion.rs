use glossa::ast::{Clause, Expr, Program, Statement, Word};
use glossa::semantic::analyze_program;

#[test]
fn test_deep_phrase_recursion() {
    // Manually build deeply nested Expr::Phrase
    // Phrase(vec![Phrase(vec![...])])
    // This bypasses the parser's recursion check which only checks source string brackets.
    //
    // NOTE: We use a depth of 500. This is enough to trigger the MAX_RECURSION_DEPTH (50)
    // check in the semantic analyzer, but small enough to avoid a stack overflow during
    // `Expr::clone()` or `Drop` (which are recursive) on standard stack sizes.
    // A depth of 20,000 would crash the test runner with a stack overflow.
    let depth = 500;
    let mut expr = Expr::NumberLiteral(1);

    for _ in 0..depth {
        expr = Expr::Phrase(vec![expr]);
    }

    // We must wrap this in a valid statement pattern that USES the value.
    // We use "ἄνθρωπος" (man) as the subject because it is a known Nominative noun.
    // "x" might be analyzed as Unknown/Object, causing "Binding without subject" error.
    // "ἄνθρωπος [deep_expr] ἔστω." (Let man be [deep_expr])
    let stmt = Statement::Regular {
        clauses: vec![Clause {
            expressions: vec![
                Expr::Word(Word::new("ἄνθρωπος")),
                expr,
                Expr::Word(Word::new("ἔστω")),
            ],
        }],
        is_query: false,
        is_propagate: false,
    };

    let program = Program {
        statements: vec![stmt],
    };

    // This should NOT panic. It should return a Result::Err.
    // Ideally "Recursion limit exceeded".
    println!("Analyzing deep phrase recursion (depth {})...", depth);
    let result = analyze_program(&program);
    println!("Analysis finished. Result: {:?}", result.is_err());

    match result {
        Ok(_) => panic!("Should have failed with recursion limit error"),
        Err(e) => {
            println!("Got expected error: {}", e);
            assert!(e.to_string().contains("Recursion limit") || e.to_string().contains("statement depth") || e.to_string().contains("expression depth"));
        }
    }
    println!("Test completed, dropping program...");
}

#[test]
fn test_wide_phrase_limit() {
    // Manually build wide Expr::Phrase (many terms)
    // Phrase(vec![1, 1, 1, ...])
    let width = 20_000;
    let terms: Vec<Expr> = (0..width).map(|_| Expr::NumberLiteral(1)).collect();
    let expr = Expr::Phrase(terms);

    let stmt = Statement::Regular {
        clauses: vec![Clause {
            expressions: vec![expr],
        }],
        is_query: false,
        is_propagate: false,
    };

    let program = Program {
        statements: vec![stmt],
    };

    // This should NOT panic.
    println!("Analyzing wide phrase (width {})...", width);
    let result = analyze_program(&program);

    // It might pass (if it's just a list of numbers without operators)
    // Or fail with "Unexpected multiple terms" if it's not a valid pattern.
    // We just want to ensure NO PANIC.
    if let Err(e) = result {
        println!("Got error (valid): {}", e);
    } else {
        println!("Analysis passed (valid)");
    }
}
