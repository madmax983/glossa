#![allow(missing_docs)]
use glossa::ast::{Clause, Expr, Statement, Word};
use glossa::semantic::analyzer::analyze_statement;
use glossa::semantic::{GlossaType, Scope};
use smol_str::SmolStr;

#[test]
fn test_parse_for_range_loop_missing_word() {
    let stmt = Statement::Regular {
        is_query: false,
        is_propagate: false,
        clauses: vec![
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word {
                        original: "ἀπὸ".into(),
                        normalized: "απο".into(),
                    }),
                    Expr::StringLiteral("not_a_word".into()), // <--- triggers the Err for start
                    Expr::Word(Word {
                        original: "ἕως".into(),
                        normalized: "εως".into(),
                    }),
                    Expr::Word(Word {
                        original: "5".into(),
                        normalized: "5".into(),
                    }),
                ])],
            },
            Clause {
                expressions: vec![Expr::Word(Word {
                    original: "i".into(),
                    normalized: "i".into(),
                })],
            },
        ],
    };

    let mut scope = Scope::new();
    let res = analyze_statement(&stmt, &mut scope);
    assert!(res.is_err());
    if let Err(e) = res {
        assert!(e.to_string().contains("Expected word for range start"));
    }
}

#[test]
fn test_parse_for_range_loop_missing_word_end() {
    let stmt = Statement::Regular {
        is_query: false,
        is_propagate: false,
        clauses: vec![
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word {
                        original: "ἀπὸ".into(),
                        normalized: "απο".into(),
                    }),
                    Expr::Word(Word {
                        original: "1".into(),
                        normalized: "1".into(),
                    }),
                    Expr::Word(Word {
                        original: "ἕως".into(),
                        normalized: "εως".into(),
                    }),
                    Expr::StringLiteral("not_a_word".into()), // <--- triggers the Err for end
                ])],
            },
            Clause {
                expressions: vec![Expr::Word(Word {
                    original: "i".into(),
                    normalized: "i".into(),
                })],
            },
        ],
    };

    let mut scope = Scope::new();
    let res = analyze_statement(&stmt, &mut scope);
    assert!(res.is_err());
    if let Err(e) = res {
        assert!(e.to_string().contains("Expected word for range end"));
    }
}

#[test]
fn test_parse_for_range_loop_variable_resolution() {
    let stmt = Statement::Regular {
        is_query: false,
        is_propagate: false,
        clauses: vec![
            Clause {
                expressions: vec![Expr::Phrase(vec![
                    Expr::Word(Word {
                        original: "ἀπὸ".into(),
                        normalized: "απο".into(),
                    }),
                    Expr::Word(Word {
                        original: "var_a".into(),
                        normalized: "var_a".into(),
                    }),
                    Expr::Word(Word {
                        original: "ἕως".into(),
                        normalized: "εως".into(),
                    }),
                    Expr::Word(Word {
                        original: "var_b".into(),
                        normalized: "var_b".into(),
                    }),
                ])],
            },
            Clause {
                expressions: vec![Expr::Word(Word {
                    original: "i".into(),
                    normalized: "i".into(),
                })],
            },
        ],
    };

    let mut scope = Scope::new();
    let var_a_str: SmolStr = "var_a".into();
    scope.define(var_a_str, GlossaType::Number);
    let var_b_str: SmolStr = "var_b".into();
    scope.define(var_b_str, GlossaType::Number);
    let res = analyze_statement(&stmt, &mut scope);
    assert!(res.is_ok());
}
