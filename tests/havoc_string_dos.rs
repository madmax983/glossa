use glossa::ast::{Statement, Clause, Expr};
use glossa::semantic::{assemble_statement, AssemblyError};
use glossa::errors::GlossaError;

#[test]
fn test_string_literal_length_limit() {
    let limit = 65536;
    let huge_string = "a".repeat(limit + 1);

    // Create a statement: "huge_string" λέγε.
    // Note: assemble_statement processes expressions. A string literal is a valid expression.
    let stmt = Statement::Regular {
        clauses: vec![Clause {
            expressions: vec![
                Expr::StringLiteral(huge_string),
            ]
        }],
        is_query: false,
        is_propagate: false,
    };

    // This should fail with LimitExceeded
    let result = assemble_statement(&stmt);

    assert!(result.is_err(), "Assembler accepted string literal longer than limit");

    match result {
        Err(GlossaError::AssemblyError(AssemblyError::LimitExceeded { resource, max })) => {
            assert_eq!(resource, "String Literal Length");
            assert_eq!(max, 65536);
        }
        _ => panic!("Expected LimitExceeded error, got {:?}", result),
    }
}
