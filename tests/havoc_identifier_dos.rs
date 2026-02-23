use glossa::ast::{Clause, Expr, Statement, Word};
use glossa::errors::GlossaError;
use glossa::semantic::{assemble_statement, AssemblyError};

#[test]
fn test_identifier_length_limit() {
    let limit = 256;
    let huge_ident = "a".repeat(limit + 1);

    // Create a statement: huge_ident.
    let stmt = Statement::Regular {
        clauses: vec![Clause {
            expressions: vec![Expr::Word(Word::new(huge_ident))],
        }],
        is_query: false,
        is_propagate: false,
    };

    // This should fail with LimitExceeded
    let result = assemble_statement(&stmt);

    assert!(
        result.is_err(),
        "Assembler accepted identifier longer than limit"
    );

    match result {
        Err(GlossaError::AssemblyError(AssemblyError::LimitExceeded { resource, max })) => {
            assert_eq!(resource, "Identifier Length");
            assert_eq!(max, 256);
        }
        _ => panic!("Expected LimitExceeded error, got {:?}", result),
    }
}

#[test]
fn test_participle_length_limit() {
    let limit = 256;
    // Create a huge participle: "aaaa..." + "μενος"
    // It must effectively fail the length check in feed_participle
    // But first, we need to ensure it IS detected as a participle.
    // The suffix "μενος" (passive participle) usually triggers detection.
    let suffix = "μενος";
    let prefix_len = limit + 1 - suffix.len() + 10; // Ensure it's comfortably over the limit
    let huge_participle = "α".repeat(prefix_len) + suffix;

    // Verify it is indeed over the limit
    assert!(huge_participle.len() > limit);

    // Create a statement: huge_participle.
    let stmt = Statement::Regular {
        clauses: vec![Clause {
            expressions: vec![Expr::Word(Word::new(huge_participle))],
        }],
        is_query: false,
        is_propagate: false,
    };

    // This should fail with LimitExceeded
    let result = assemble_statement(&stmt);

    assert!(
        result.is_err(),
        "Assembler accepted participle longer than limit"
    );

    match result {
        Err(GlossaError::AssemblyError(AssemblyError::LimitExceeded { resource, max })) => {
            assert_eq!(resource, "Identifier Length");
            assert_eq!(max, 256);
        }
        _ => panic!("Expected LimitExceeded error, got {:?}", result),
    }
}
