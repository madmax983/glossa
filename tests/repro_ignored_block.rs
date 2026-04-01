#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;
use glossa::semantic::{AnalyzedExprKind, AnalyzedStatement};

#[test]
fn test_block_in_binding_ignored() {
    // Case 1: Block as direct value
    // x { 1. } ἔστω.
    // Expected: x = 1
    // Actual (suspected): x = 0 (block ignored)

    let source = "ξ { 1. } ἔστω.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast).unwrap();

    if let AnalyzedStatement::Binding { name, value, .. } = &result.statements[0] {
        assert_eq!(name, "ξ");
        match value.expr {
            AnalyzedExprKind::NumberLiteral(n) => {
                assert_eq!(n, 1, "Block {{ 1. }} should evaluate to 1, but got {}", n);
            }
            _ => panic!("Expected NumberLiteral"),
        }
    } else {
        panic!("Expected Binding statement");
    }
}

#[test]
fn test_multi_statement_block_ignored() {
    // Case 2: Multi-statement block in nested phrase
    // x ({ 1. 2. }) ἔστω.
    // Expected: Error (or x = 2 if we supported blocks, but we don't really)
    // Actual (suspected): x = 1 (second statement ignored)

    let source = "ξ ({ 1. 2. }) ἔστω.";
    let ast = parse(source).unwrap();

    // We expect this to fail with "Multiple statements in block expression not supported"
    // or similar, instead of silently returning 1.
    let result = analyze_program(&ast);

    if let Ok(program) = result {
        if let AnalyzedStatement::Binding { value, .. } = &program.statements[0] {
            match value.expr {
                AnalyzedExprKind::NumberLiteral(n) => {
                    println!("Got number: {}", n);
                    assert_ne!(
                        n, 1,
                        "Should not return first statement of multi-statement block. It ignored the second statement!"
                    );
                }
                _ => {
                    println!("Got other expression: {:?}", value);
                }
            }
        }
    } else {
        println!("Got error: {:?}", result.err());
    }
}
