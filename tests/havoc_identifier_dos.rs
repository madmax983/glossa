use glossa::ast::{Statement, Clause, Expr, Word};
use glossa::semantic::{assemble_statement, AssemblyError};
use glossa::errors::GlossaError;

#[test]
fn test_identifier_length_limit() {
    let limit = 256;
    let huge_ident = "a".repeat(limit + 1);

    // Create a statement: huge_ident.
    let stmt = Statement::Regular {
        clauses: vec![Clause {
            expressions: vec![
                Expr::Word(Word::new(huge_ident)),
            ]
        }],
        is_query: false,
        is_propagate: false,
    };

    // This should fail with LimitExceeded
    let result = assemble_statement(&stmt);

    assert!(result.is_err(), "Assembler accepted identifier longer than limit");

    match result {
        Err(GlossaError::AssemblyError(AssemblyError::LimitExceeded { resource, max })) => {
            assert_eq!(resource, "Identifier Length");
            assert_eq!(max, 256);
        }
        _ => panic!("Expected LimitExceeded error, got {:?}", result),
    }
}
