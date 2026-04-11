use crate::ast::{Clause, Expr, Statement, Word};
use crate::semantic::Scope;
use crate::semantic::analyze_statement;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_for_range_loop_empty_range_clause() {
        let stmt = Statement::Regular {
            is_query: false,
            is_propagate: false,
            clauses: vec![
                Clause {
                    expressions: vec![Expr::Word(Word {
                        original: "ἀπὸ".into(),
                        normalized: "απο".into(),
                    })],
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
            assert!(e.to_string().contains("Expected phrase in for range"));
        }
    }

    #[test]
    fn test_parse_for_range_loop_too_few_words() {
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
            assert!(
                e.to_string()
                    .contains("For range needs: ἀπὸ start μέχρι/ἕως end")
            );
        }
    }

    #[test]
    fn test_match_expression_empty_clauses() {
        let stmt = Statement::Regular {
            is_query: false,
            is_propagate: false,
            clauses: vec![Clause {
                expressions: vec![Expr::Word(Word {
                    original: "κατά".into(),
                    normalized: "κατα".into(),
                })],
            }],
        };

        let mut scope = Scope::new();
        let res = analyze_statement(&stmt, &mut scope);
        assert!(res.is_err());
        if let Err(e) = res {
            assert!(
                e.to_string()
                    .contains("Match expression needs at least one arm")
                    || e.to_string().contains("Empty condition in conditional")
            );
        }
    }

    #[test]
    fn test_match_expression_pattern_without_body() {
        let stmt = Statement::Regular {
            is_query: false,
            is_propagate: false,
            clauses: vec![Clause {
                expressions: vec![
                    Expr::Word(Word {
                        original: "κατά".into(),
                        normalized: "κατα".into(),
                    }),
                    Expr::Word(Word {
                        original: "pat".into(),
                        normalized: "pat".into(),
                    }),
                ],
            }],
        };

        let mut scope = Scope::new();
        let res = analyze_statement(&stmt, &mut scope);
        assert!(res.is_err());
        if let Err(e) = res {
            assert!(
                e.to_string().contains("Match pattern without body")
                    || e.to_string().contains("Empty condition in conditional")
            );
        }
    }

    #[test]
    fn test_match_expression_empty_arm_body() {
        let stmt = Statement::Regular {
            is_query: false,
            is_propagate: false,
            clauses: vec![
                Clause {
                    expressions: vec![
                        Expr::Word(Word {
                            original: "κατά".into(),
                            normalized: "κατα".into(),
                        }),
                        Expr::Word(Word {
                            original: "pat".into(),
                            normalized: "pat".into(),
                        }),
                    ],
                },
                Clause {
                    expressions: vec![],
                },
            ],
        };

        let mut scope = Scope::new();
        let res = analyze_statement(&stmt, &mut scope);
        assert!(res.is_err());
        if let Err(e) = res {
            assert!(
                e.to_string().contains("Empty match arm body")
                    || e.to_string().contains("Empty condition in conditional")
            );
        }
    }

    #[test]
    fn test_parse_while_loop_missing_body() {
        let stmt = Statement::Regular {
            is_query: false,
            is_propagate: false,
            clauses: vec![
                Clause {
                    expressions: vec![
                        Expr::Word(Word {
                            original: "ἕως".into(),
                            normalized: "εως".into(),
                        }),
                        Expr::Word(Word {
                            original: "cond".into(),
                            normalized: "cond".into(),
                        }),
                    ],
                }, // Missing body clause
            ],
        };

        let mut scope = Scope::new();
        let res = analyze_statement(&stmt, &mut scope);
        assert!(res.is_err());
        if let Err(e) = res {
            assert!(
                e.to_string()
                    .contains("While loop needs at least 2 clauses: condition and body")
            );
        }
    }

    #[test]
    fn test_parse_conditional_missing_body() {
        let stmt = Statement::Regular {
            is_query: false,
            is_propagate: false,
            clauses: vec![
                Clause {
                    expressions: vec![
                        Expr::Word(Word {
                            original: "εἰ".into(),
                            normalized: "ει".into(),
                        }),
                        Expr::Word(Word {
                            original: "cond".into(),
                            normalized: "cond".into(),
                        }),
                    ],
                }, // Missing body clause
            ],
        };

        let mut scope = Scope::new();
        let res = analyze_statement(&stmt, &mut scope);
        assert!(res.is_err());
        if let Err(e) = res {
            assert!(
                e.to_string()
                    .contains("Conditional needs at least 2 clauses: condition and body")
            );
        }
    }

    #[test]
    fn test_parse_conditional_empty_then_body() {
        let stmt = Statement::Regular {
            is_query: false,
            is_propagate: false,
            clauses: vec![
                Clause {
                    expressions: vec![
                        Expr::Word(Word {
                            original: "εἰ".into(),
                            normalized: "ει".into(),
                        }),
                        Expr::Word(Word {
                            original: "cond".into(),
                            normalized: "cond".into(),
                        }),
                    ],
                },
                Clause {
                    expressions: vec![],
                },
            ],
        };

        let mut scope = Scope::new();
        let res = analyze_statement(&stmt, &mut scope);
        assert!(res.is_err());
        if let Err(e) = res {
            assert!(e.to_string().contains("Empty condition in conditional"));
        }
    }
}
