use glossa::ast::{Clause, Expr, Program, Statement, Word};
use glossa::semantic::analyze_program;

#[test]
fn test_deep_phrase_recursion() {
    // Manually build deeply nested Expr::Phrase
    // Phrase(vec![Phrase(vec![...])])
    // This bypasses the parser's recursion check which only checks source string brackets.
    let depth = 20_000;
    let mut expr = Expr::NumberLiteral(1);

    for _ in 0..depth {
        expr = Expr::Phrase(vec![expr]);
    }

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
