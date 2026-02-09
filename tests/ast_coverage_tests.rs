use glossa::ast::{Clause, Expr, Statement, TestDecl, Word};

#[test]
fn test_word_new() {
    let w = Word::new("Ἀθῆναι");
    assert_eq!(w.original, "Ἀθῆναι");
    assert_eq!(w.normalized, "αθηναι");
}

#[test]
fn test_statement_methods_regular() {
    let clause = Clause {
        expressions: vec![Expr::NumberLiteral(1)],
    };
    let stmt = Statement::Regular {
        clauses: vec![clause.clone()],
        is_query: false,
        is_propagate: false,
    };

    assert!(!stmt.is_query());
    assert!(!stmt.is_propagate());
    assert_eq!(stmt.clauses().len(), 1);
    assert_eq!(stmt.expressions().count(), 1);
}

#[test]
fn test_statement_methods_query() {
    let stmt = Statement::Regular {
        clauses: vec![],
        is_query: true,
        is_propagate: false,
    };
    assert!(stmt.is_query());
    assert!(!stmt.is_propagate());
}

#[test]
fn test_statement_methods_propagate() {
    let stmt = Statement::Regular {
        clauses: vec![],
        is_query: false,
        is_propagate: true,
    };
    assert!(!stmt.is_query());
    assert!(stmt.is_propagate());
}

#[test]
fn test_statement_methods_non_regular() {
    let decl = TestDecl {
        name: "test".to_string(),
        body: vec![],
    };
    let stmt = Statement::TestDeclaration(decl);

    assert!(!stmt.is_query());
    assert!(!stmt.is_propagate());
    assert!(stmt.clauses().is_empty());
    assert_eq!(stmt.expressions().count(), 0);
}
