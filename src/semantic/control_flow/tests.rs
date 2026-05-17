#![allow(unused_imports)]
use super::*;
use crate::ast::{Clause, Expr, Statement, Word};

#[test]
fn test_parse_match_expression_missing_clauses() {
    let mut scope = Scope::new();
    // Statement with empty clauses
    let stmt = Statement::Regular {
        clauses: vec![],
        is_query: false,
        is_propagate: false,
    };
    // We know parse_match_expression just looks at clauses() which is empty here.
    let err = parse_match_expression(&stmt, &mut scope);
    assert!(err.is_err());
}

#[test]
fn test_parse_match_pattern_uncovered() {
    let mut scope = Scope::new();

    // Phrase with boolean literal
    let phrase_bool = Expr::Phrase(vec![Expr::BooleanLiteral(true)]);
    let err = parse_match_pattern(&phrase_bool, &mut scope);
    assert!(err.is_err());

    // Number literal directly
    let num_expr = Expr::NumberLiteral(5);
    let err = parse_match_pattern(&num_expr, &mut scope);
    assert!(err.is_err());

    // Phrase with undefined word
    let phrase_undef = Expr::Phrase(vec![Expr::Word(Word::new("ἀγνώστου"))]);
    let err = parse_match_pattern(&phrase_undef, &mut scope);
    assert!(err.is_err());

    // Phrase with wild-card "ἄλλο"
    let phrase_wildcard = Expr::Phrase(vec![Expr::Word(Word::new("ἄλλο"))]);
    let res = parse_match_pattern(&phrase_wildcard, &mut scope).unwrap();
    if let AnalyzedExprKind::BooleanLiteral(b) = res.expr {
        assert!(b);
    } else {
        panic!("Expected boolean literal");
    }

    // Phrase with numeral "δύο"
    let phrase_numeral = Expr::Phrase(vec![Expr::Word(Word::new("δύο"))]);
    let res = parse_match_pattern(&phrase_numeral, &mut scope).unwrap();
    if let AnalyzedExprKind::NumberLiteral(n) = res.expr {
        assert_eq!(n, 2);
    } else {
        panic!("Expected number literal");
    }

    // Undefined variable check
    let word_undef = Expr::Word(Word::new("ἀγνώστου"));
    let err = parse_match_pattern(&word_undef, &mut scope);
    assert!(err.is_err());

    // Phrase with boolean literal at the beginning
    let phrase_bool = Expr::Phrase(vec![Expr::BooleanLiteral(true)]);
    let err = parse_match_pattern(&phrase_bool, &mut scope);
    assert!(err.is_err());

    // Match string literal directly
    let str_expr = Expr::StringLiteral("test".to_string());
    let err = parse_match_pattern(&str_expr, &mut scope);
    assert!(err.is_err());

    // Match phrase with string literal
    let phrase_str = Expr::Phrase(vec![Expr::StringLiteral("test".to_string())]);
    let err = parse_match_pattern(&phrase_str, &mut scope);
    assert!(err.is_err());

    // Match phrase with NumberLiteral
    let phrase_num = Expr::Phrase(vec![Expr::NumberLiteral(5)]);
    let err = parse_match_pattern(&phrase_num, &mut scope);
    assert!(err.is_err());

    // Empty phrase
    let empty_phrase = Expr::Phrase(vec![]);
    let err = parse_match_pattern(&empty_phrase, &mut scope);
    assert!(err.is_err());
}

#[test]
fn test_parse_return_expression_boolean() {
    let scope = Scope::new();
    let clause = Clause {
        expressions: vec![Expr::Phrase(vec![
            Expr::Word(Word::new("δός")),
            Expr::BooleanLiteral(true),
        ])],
    };

    let result = parse_return_expression(&clause, &scope).unwrap();
    match result.expr {
        AnalyzedExprKind::BooleanLiteral(true) => (),
        _ => panic!("Expected BooleanLiteral(true)"),
    }
    assert_eq!(result.glossa_type, GlossaType::Boolean);
}

#[test]
fn test_parse_return_expression_string() {
    let scope = Scope::new();
    let clause = Clause {
        expressions: vec![Expr::Phrase(vec![
            Expr::Word(Word::new("δός")),
            Expr::StringLiteral("test".to_string()),
        ])],
    };

    let result = parse_return_expression(&clause, &scope).unwrap();
    match result.expr {
        AnalyzedExprKind::StringLiteral(s) if s == "test" => (),
        _ => panic!("Expected StringLiteral"),
    }
    assert_eq!(result.glossa_type, GlossaType::String);
}

#[test]
fn test_parse_return_expression_number() {
    let scope = Scope::new();
    let clause = Clause {
        expressions: vec![Expr::Phrase(vec![
            Expr::Word(Word::new("δός")),
            Expr::NumberLiteral(42),
        ])],
    };

    let result = parse_return_expression(&clause, &scope).unwrap();
    match result.expr {
        AnalyzedExprKind::NumberLiteral(42) => (),
        _ => panic!("Expected NumberLiteral(42)"),
    }
    assert_eq!(result.glossa_type, GlossaType::Number);
}

#[test]
fn test_parse_return_expression_variable() {
    let mut scope = Scope::new();
    scope.define("foo", GlossaType::String);
    let clause = Clause {
        expressions: vec![Expr::Phrase(vec![
            Expr::Word(Word::new("δός")),
            Expr::Word(Word::new("foo")),
        ])],
    };

    let result = parse_return_expression(&clause, &scope).unwrap();
    match result.expr {
        AnalyzedExprKind::Variable(v) if v == "foo" => (),
        _ => panic!("Expected Variable(foo)"),
    }
    assert_eq!(result.glossa_type, GlossaType::String);
}

#[test]
fn test_parse_return_statement_empty_clauses() {
    let mut scope = Scope::new();
    let stmt = Statement::Regular {
        clauses: vec![],
        is_query: false,
        is_propagate: false,
    };

    let result = parse_return_statement(&stmt, &mut scope).unwrap();
    match result {
        Some(AnalyzedStatement::Return { value: None }) => (),
        _ => panic!("Expected Return with None"),
    }
}

#[test]
fn test_for_iteration_error_not_word() {
    let mut scope = Scope::new();

    let stmt = Statement::Regular {
        clauses: vec![
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word::new("δια")),
                    Expr::NumberLiteral(5),
                ])],
            },
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word::new("ν")),
                    Expr::Word(Word::new("λεγε")),
                ])],
            },
        ],
        is_query: false,
        is_propagate: false,
    };

    let result = analyze_control_flow(&stmt, &mut scope);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Expected word for collection")
    );
}

#[test]
fn test_for_iteration_error_missing_collection() {
    let mut scope = Scope::new();

    let stmt = Statement::Regular {
        clauses: vec![
            Clause {
                expressions: vec![Expr::Phrase(vec![Expr::Word(Word::new("δια"))])],
            },
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word::new("ν")),
                    Expr::Word(Word::new("λεγε")),
                ])],
            },
        ],
        is_query: false,
        is_propagate: false,
    };

    let result = analyze_control_flow(&stmt, &mut scope);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("For iteration needs: διὰ collection")
    );
}

#[test]
fn test_for_iteration_error_not_phrase() {
    let mut scope = Scope::new();

    // This requires testing parse_for_iteration_loop directly or bypassing analyze_control_flow
    // Since analyze_control_flow filters on get_first_word (which expects a Phrase),
    // we call the inner function.
    let stmt = Statement::Regular {
        clauses: vec![
            Clause {
                expressions: vec![Expr::NumberLiteral(10)], // Not a phrase
            },
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word::new("ν")),
                    Expr::Word(Word::new("λεγε")),
                ])],
            },
        ],
        is_query: false,
        is_propagate: false,
    };

    let result = parse_for_iteration_loop(&stmt, &mut scope);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Expected phrase in for iteration")
    );
}

#[test]
fn test_check_else_pattern_not_phrase() {
    let expr = Expr::NumberLiteral(42);
    assert!(!super::check_else_pattern_in_expression(&expr));
}

#[test]
fn test_parse_while_loop_missing_body() {
    let mut scope = Scope::new();

    // ἕως x == 5
    let stmt = Statement::Regular {
        clauses: vec![Clause {
            expressions: vec![Expr::Phrase(vec![
                Expr::Word(Word::new("εως")),
                Expr::Word(Word::new("ξ")),
                Expr::Word(Word::new("εστι")),
                Expr::Word(Word::new("ισον")),
                Expr::NumberLiteral(5),
            ])],
        }],
        is_query: false,
        is_propagate: false,
    };

    let result = analyze_control_flow(&stmt, &mut scope);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("While loop needs at least 2 clauses: condition and body")
    );
}

#[test]
fn test_parse_while_loop_success() {
    let mut scope = Scope::new();
    // pre-define ξ so the expression analyzer knows it
    scope.define("ξ".to_string(), GlossaType::Number);

    // ἕως ξ ἴσον 5· «γεια» λέγε
    let stmt = Statement::Regular {
        clauses: vec![
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word::new("εως")),
                    Expr::Word(Word::new("ξ")),
                    Expr::Word(Word::new("εστι")),
                    Expr::Word(Word::new("ισον")),
                    Expr::NumberLiteral(5),
                ])],
            },
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::StringLiteral("γεια".to_string()),
                    Expr::Word(Word::new("λεγε")),
                ])],
            },
        ],
        is_query: false,
        is_propagate: false,
    };

    let result = analyze_control_flow(&stmt, &mut scope);
    assert!(result.is_ok());
    let analyzed = result.unwrap().unwrap();

    match analyzed {
        AnalyzedStatement::While { condition, body } => {
            // Assert condition
            assert_eq!(condition.glossa_type, GlossaType::Boolean);
            // Assert body
            assert!(!body.is_empty());
        }
        _ => panic!("Expected While statement"),
    }
}

#[test]
fn test_parse_for_range_missing_body() {
    let mut scope = Scope::new();

    // ἀπὸ 1 μέχρι 5
    let stmt = Statement::Regular {
        clauses: vec![Clause {
            expressions: vec![Expr::Phrase(vec![
                Expr::Word(Word::new("απο")),
                Expr::NumberLiteral(1),
                Expr::Word(Word::new("μεχρι")),
                Expr::NumberLiteral(5),
            ])],
        }],
        is_query: false,
        is_propagate: false,
    };

    let result = analyze_control_flow(&stmt, &mut scope);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("For loop needs at least 2 clauses: range and body")
    );
}

#[test]
fn test_parse_conditional_max_depth() {
    let mut scope = Scope::new();
    let stmt = Statement::Regular {
        clauses: vec![
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word::new("εαν")),
                    Expr::NumberLiteral(1),
                ])],
            },
            Clause {
                expressions: vec![Expr::NumberLiteral(2)],
            },
        ],
        is_query: false,
        is_propagate: false,
    };

    let result = parse_conditional(&stmt, &mut scope, crate::limits::MAX_CONTROL_FLOW_DEPTH + 1);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Control flow depth")
    );
}

#[test]
fn test_parse_conditional_else_branch() {
    let mut scope = Scope::new();
    // εαν 1, λεγε "γεια"
    // ει δε μη, λεγε "αντιο"
    let stmt = Statement::Regular {
        clauses: vec![
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word::new("εαν")),
                    Expr::NumberLiteral(1),
                ])],
            },
            Clause {
                expressions: vec![
                    Expr::Phrase(vec![
                        Expr::StringLiteral("γεια".to_string()),
                        Expr::Word(Word::new("λεγε")),
                    ]),
                    Expr::Phrase(vec![
                        Expr::Word(Word::new("ει")),
                        Expr::Word(Word::new("δε")),
                        Expr::Word(Word::new("μη")),
                    ]),
                ],
            },
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::StringLiteral("αντιο".to_string()),
                    Expr::Word(Word::new("λεγε")),
                ])],
            },
        ],
        is_query: false,
        is_propagate: false,
    };
    let result = parse_conditional(&stmt, &mut scope, 0);
    assert!(result.is_ok());
    let stmt = result.unwrap().unwrap();
    if let AnalyzedStatement::If {
        condition: _,
        then_body: _,
        else_body,
    } = stmt
    {
        assert!(else_body.is_some());
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_parse_conditional_elif_branch() {
    let mut scope = Scope::new();
    // εαν 1, λεγε "1"
    // εαν 2, λεγε "2"
    let stmt = Statement::Regular {
        clauses: vec![
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word::new("εαν")),
                    Expr::NumberLiteral(1),
                ])],
            },
            Clause {
                expressions: vec![
                    Expr::Phrase(vec![
                        Expr::StringLiteral("1".to_string()),
                        Expr::Word(Word::new("λεγε")),
                    ]),
                    Expr::Phrase(vec![Expr::Word(Word::new("εαν")), Expr::NumberLiteral(2)]),
                ],
            },
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::StringLiteral("2".to_string()),
                    Expr::Word(Word::new("λεγε")),
                ])],
            },
        ],
        is_query: false,
        is_propagate: false,
    };
    let result = parse_conditional(&stmt, &mut scope, 0);
    assert!(result.is_ok());
    let stmt = result.unwrap().unwrap();
    if let AnalyzedStatement::If {
        condition: _,
        then_body: _,
        else_body,
    } = stmt
    {
        assert!(else_body.is_some());
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_parse_conditional_binding_condition() {
    let mut scope = Scope::new();
    scope.define("χ".to_string(), GlossaType::Number);

    // εαν χ ισον 5, λεγε "γεια"
    let stmt = Statement::Regular {
        clauses: vec![
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word::new("εαν")),
                    Expr::Word(Word::new("χ")),
                    Expr::Word(Word::new("εστι")),
                    Expr::Word(Word::new("ισον")),
                    Expr::NumberLiteral(5),
                ])],
            },
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::StringLiteral("γεια".to_string()),
                    Expr::Word(Word::new("λεγε")),
                ])],
            },
        ],
        is_query: false,
        is_propagate: false,
    };
    let result = parse_conditional(&stmt, &mut scope, 0);
    assert!(result.is_ok());
    let stmt = result.unwrap().unwrap();
    if let AnalyzedStatement::If {
        condition,
        then_body: _,
        else_body: _,
    } = stmt
    {
        // Expected condition to be binop ==
        if let AnalyzedExprKind::BinOp { op, .. } = condition.expr {
            assert_eq!(op, crate::morphology::lexicon::BinaryOp::Eq);
        } else {
            panic!("Expected condition to be BinOp");
        }
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_check_else_pattern_in_expression_true() {
    let expr = Expr::Phrase(vec![
        Expr::Word(Word::new("ει")),
        Expr::Word(Word::new("δε")),
        Expr::Word(Word::new("μη")),
        Expr::Word(Word::new("λεγε")),
    ]);
    assert!(check_else_pattern_in_expression(&expr));
}

#[test]
fn test_check_conditional_start_true_phrase() {
    let expr = Expr::Phrase(vec![
        Expr::Word(Word::new("ει")),
        Expr::Word(Word::new("λεγε")),
    ]);
    assert!(check_conditional_start(&expr));
}

#[test]
fn test_check_conditional_start_true_word() {
    let expr = Expr::Word(Word::new("εαν"));
    assert!(check_conditional_start(&expr));
}
