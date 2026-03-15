use glossa::ast::{Clause, Expr, Statement, Word};
use glossa::semantic::assemble_statement;

// Helper to assert that assembly returns a LimitExceeded error
fn assert_limit_exceeded(stmt: &Statement, resource_name: &str) {
    let result = assemble_statement(stmt);
    match result {
        Err(e) => {
            let err_str = format!("{:?}", e);
            if !err_str.contains("LimitExceeded") {
                panic!("Expected LimitExceeded for {}, got: {:?}", resource_name, e);
            }
            if !err_str.contains(resource_name) {
                panic!(
                    "Expected LimitExceeded for {}, got error for wrong resource: {:?}",
                    resource_name, e
                );
            }
        }
        Ok(s) => {
            // Panic with details
            panic!(
                "Expected LimitExceeded for {}, got Ok with statement: {:?}",
                resource_name, s
            );
        }
    }
}

// Helper to assert that ANY LimitExceeded error is returned
// This is useful for ambiguous words that might hit different limits (e.g. Adjective vs Nominative)
fn assert_any_limit_exceeded(stmt: &Statement) {
    let result = assemble_statement(stmt);
    match result {
        Err(e) => {
            let err_str = format!("{:?}", e);
            if !err_str.contains("LimitExceeded") {
                panic!("Expected LimitExceeded, got: {:?}", e);
            }
        }
        Ok(s) => {
            panic!("Expected LimitExceeded, got Ok with statement: {:?}", s);
        }
    }
}

#[test]
fn test_limit_nominatives() {
    let mut expressions = Vec::new();
    // 300 nominatives (MAX=256)
    // Use "ἄνθρωπος" (man) - known nominative
    for _ in 0..300 {
        expressions.push(Expr::Word(Word::new("ἄνθρωπος")));
    }

    let stmt = Statement::Regular {
        clauses: vec![Clause { expressions }],
        is_query: false,
        is_propagate: false,
    };

    assert_limit_exceeded(&stmt, "Nominatives");
}

#[test]
fn test_limit_literals() {
    let mut expressions = Vec::new();
    // 1100 literals (MAX=1024)
    for i in 0..1100 {
        expressions.push(Expr::NumberLiteral(i as i64));
    }

    let stmt = Statement::Regular {
        clauses: vec![Clause { expressions }],
        is_query: false,
        is_propagate: false,
    };

    assert_limit_exceeded(&stmt, "Literals");
}

#[test]
fn test_limit_arrays() {
    let mut expressions = Vec::new();
    // 300 arrays (MAX=256)
    for _ in 0..300 {
        expressions.push(Expr::ArrayLiteral(vec![]));
    }

    let stmt = Statement::Regular {
        clauses: vec![Clause { expressions }],
        is_query: false,
        is_propagate: false,
    };

    assert_limit_exceeded(&stmt, "Arrays");
}

#[test]
fn test_limit_blocks() {
    let mut expressions = Vec::new();
    // 300 blocks (MAX=256)
    for _ in 0..300 {
        expressions.push(Expr::Block(vec![]));
    }

    let stmt = Statement::Regular {
        clauses: vec![Clause { expressions }],
        is_query: false,
        is_propagate: false,
    };

    assert_limit_exceeded(&stmt, "Blocks");
}

#[test]
fn test_limit_nested_phrases() {
    let mut expressions = Vec::new();
    // 300 nested phrases (MAX=256)
    for _ in 0..300 {
        // Phrase inside Phrase triggers nested phrase logic
        expressions.push(Expr::Phrase(vec![Expr::Phrase(vec![])]));
    }

    let stmt = Statement::Regular {
        clauses: vec![Clause { expressions }],
        is_query: false,
        is_propagate: false,
    };

    assert_limit_exceeded(&stmt, "Nested Phrases");
}

#[test]
fn test_limit_genitives() {
    let mut expressions = Vec::new();
    // 300 genitives (MAX=256)
    // "λόγου" (of word) - known genitive
    for _ in 0..300 {
        expressions.push(Expr::Word(Word::new("λόγου")));
    }

    let stmt = Statement::Regular {
        clauses: vec![Clause { expressions }],
        is_query: false,
        is_propagate: false,
    };

    assert_limit_exceeded(&stmt, "Genitives");
}

#[test]
fn test_limit_adjectives() {
    let mut expressions = Vec::new();
    // 1100 adjectives (MAX=1024)
    // "καλός" (good) - known adjective, but may be ambiguous with Noun
    for _ in 0..1100 {
        expressions.push(Expr::Word(Word::new("καλός")));
    }

    let stmt = Statement::Regular {
        clauses: vec![Clause { expressions }],
        is_query: false,
        is_propagate: false,
    };

    // Allow either Nominatives or Adjectives limit, acknowledging ambiguity
    assert_any_limit_exceeded(&stmt);
}

#[test]
fn test_limit_operators() {
    let mut expressions = Vec::new();
    // 300 operators (MAX=256)
    // "καί" (and) - known conjunction/operator
    for _ in 0..300 {
        expressions.push(Expr::Word(Word::new("καί")));
    }

    let stmt = Statement::Regular {
        clauses: vec![Clause { expressions }],
        is_query: false,
        is_propagate: false,
    };

    assert_limit_exceeded(&stmt, "Operators");
}

#[test]
fn test_limit_property_accesses() {
    let mut expressions = Vec::new();
    // 300 property accesses (MAX=256)
    // "μῆκος" (length) - special property noun

    for _ in 0..300 {
        // Subject: "ἄνθρωπος"
        expressions.push(Expr::Word(Word::new("ἄνθρωπος")));
        // Property: "μῆκος"
        expressions.push(Expr::Word(Word::new("μῆκος")));
    }

    let stmt = Statement::Regular {
        clauses: vec![Clause { expressions }],
        is_query: false,
        is_propagate: false,
    };

    assert_limit_exceeded(&stmt, "Property Accesses");
}

#[test]
fn test_limit_index_accesses_via_ordinals() {
    let mut expressions = Vec::new();
    // 300 index accesses (MAX=256)
    // "πρῶτον" (first) - ordinal adjective

    for _ in 0..300 {
        // Subject: "ἄνθρωπος"
        expressions.push(Expr::Word(Word::new("ἄνθρωπος")));
        // Ordinal: "πρῶτον"
        expressions.push(Expr::Word(Word::new("πρῶτον")));
    }

    let stmt = Statement::Regular {
        clauses: vec![Clause { expressions }],
        is_query: false,
        is_propagate: false,
    };

    assert_limit_exceeded(&stmt, "Index Accesses");
}

#[test]
fn test_limit_participles() {
    let mut expressions = Vec::new();
    // 300 participles (MAX=256)
    // "λύων" (loosing) - present active participle of λύω
    for _ in 0..300 {
        expressions.push(Expr::Word(Word::new("λύων")));
    }

    let stmt = Statement::Regular {
        clauses: vec![Clause { expressions }],
        is_query: false,
        is_propagate: false,
    };

    assert_limit_exceeded(&stmt, "Participles");
}

#[test]
fn test_limit_unwraps() {
    let mut expressions = Vec::new();
    // 300 unwraps (MAX=256)
    // Need UnaryOp::Unwrap expr
    for _ in 0..300 {
        expressions.push(Expr::UnaryOp {
            op: glossa::ast::UnaryOperator::Unwrap,
            operand: Box::new(Expr::NumberLiteral(1)),
        });
    }

    let stmt = Statement::Regular {
        clauses: vec![Clause { expressions }],
        is_query: false,
        is_propagate: false,
    };

    assert_limit_exceeded(&stmt, "Unwraps");
}
