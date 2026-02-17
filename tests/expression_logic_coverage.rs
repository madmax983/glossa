#[cfg(test)]
mod tests {
    use glossa::ast::{Expr, Word};
    use glossa::morphology::lexicon::BinaryOp;
    use glossa::semantic::expressions::analyze_argument_expr;
    use glossa::semantic::{AnalyzedExprKind, GlossaType, Scope};

    #[test]
    fn test_subject_operator_literal_expression() {
        let mut scope = Scope::new();
        scope.define("λογον", GlossaType::Number);

        let expr = Expr::Phrase(vec![
            Expr::Word(Word::new("λόγον")),
            Expr::NumberLiteral(1),
            Expr::Word(Word::new("ἄθροισμα")),
        ]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::BinOp { left, op, right } => {
                assert!(matches!(left.expr, AnalyzedExprKind::Variable(name) if name == "λογον"));
                assert!(matches!(right.expr, AnalyzedExprKind::NumberLiteral(1)));
                assert_eq!(op, BinaryOp::Add);
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_subject_operator_object_expression() {
        let mut scope = Scope::new();
        scope.define("λογον", GlossaType::Number);

        let expr = Expr::Phrase(vec![
            Expr::Word(Word::new("λόγον")),
            Expr::Word(Word::new("λόγον")),
            Expr::Word(Word::new("ἄθροισμα")),
        ]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::BinOp { left, op, right } => {
                assert!(matches!(left.expr, AnalyzedExprKind::Variable(name) if name == "λογον"));
                assert!(matches!(right.expr, AnalyzedExprKind::Variable(name) if name == "λογον"));
                assert_eq!(op, BinaryOp::Add);
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_literal_operator_object_expression() {
        let mut scope = Scope::new();
        scope.define("μηλον", GlossaType::Number);

        let expr = Expr::Phrase(vec![
            Expr::NumberLiteral(1),
            Expr::Word(Word::new("μήλον")),
            Expr::Word(Word::new("ἄθροισμα")),
        ]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::BinOp { left, op, right } => {
                if let AnalyzedExprKind::NumberLiteral(n) = left.expr {
                    assert_eq!(n, 1);
                    if let AnalyzedExprKind::Variable(name) = right.expr {
                        assert_eq!(name, "μηλον");
                    } else {
                        panic!("Right should be variable");
                    }
                } else if let AnalyzedExprKind::Variable(name) = left.expr {
                    assert_eq!(name, "μηλον");
                    if let AnalyzedExprKind::NumberLiteral(n) = right.expr {
                        assert_eq!(n, 1);
                    } else {
                        panic!("Right should be literal");
                    }
                } else {
                    panic!("Unexpected operands");
                }
                assert_eq!(op, BinaryOp::Add);
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_nested_phrase_expression() {
        let scope = Scope::new();
        let inner = Expr::Phrase(vec![Expr::NumberLiteral(1)]);
        let outer = Expr::Phrase(vec![inner]);

        let result = analyze_argument_expr(&outer, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::NumberLiteral(1) => {}
            _ => panic!("Expected NumberLiteral(1)"),
        }
    }

    #[test]
    fn test_just_subject_expression() {
        let mut scope = Scope::new();
        scope.define("λογον", GlossaType::Number);

        let expr = Expr::Phrase(vec![Expr::Word(Word::new("λόγον"))]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::Variable(name) => assert_eq!(name, "λογον"),
            _ => panic!("Expected Variable"),
        }
    }

    #[test]
    fn test_mixed_nested_expression() {
        let scope = Scope::new();
        let one = Expr::Phrase(vec![Expr::NumberLiteral(1)]);
        let two = Expr::NumberLiteral(2);
        let plus = Expr::Word(Word::new("ἄθροισμα"));

        let expr = Expr::Phrase(vec![one, two, plus]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::BinOp { left, op, right } => {
                assert!(matches!(left.expr, AnalyzedExprKind::NumberLiteral(1)));
                assert!(matches!(right.expr, AnalyzedExprKind::NumberLiteral(2)));
                assert_eq!(op, BinaryOp::Add);
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_variable_fallback_original() {
        let mut scope = Scope::new();
        scope.define("δωρα", GlossaType::Number);

        let expr = Expr::Phrase(vec![Expr::Word(Word::new("δῶρα"))]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::Variable(name) => assert_eq!(name, "δωρα"),
            _ => panic!("Expected Variable"),
        }
    }

    #[test]
    fn test_subject_nominative_expression() {
        let mut scope = Scope::new();
        scope.define("α", GlossaType::Number);
        scope.define("β", GlossaType::Number);

        let expr = Expr::Phrase(vec![
            Expr::Word(Word::new("Α")),
            Expr::Word(Word::new("Β")),
            Expr::Word(Word::new("ἄθροισμα")),
        ]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::BinOp { left, op, right } => {
                assert!(matches!(left.expr, AnalyzedExprKind::Variable(name) if name == "α"));
                assert!(matches!(right.expr, AnalyzedExprKind::Variable(name) if name == "β"));
                assert_eq!(op, BinaryOp::Add);
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_participle_verb_lemma_fallback() {
        let mut scope = Scope::new();
        scope.define("τρεχω", GlossaType::Number);

        let expr = Expr::Phrase(vec![Expr::Word(Word::new("τρέχων"))]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::Variable(name) => assert_eq!(name, "τρεχω"),
            _ => panic!("Expected Variable"),
        }
    }

    #[test]
    fn test_subject_participle_expression() {
        let mut scope = Scope::new();
        scope.define("α", GlossaType::Number);
        scope.define("τρεχων", GlossaType::Number);

        let expr = Expr::Phrase(vec![
            Expr::Word(Word::new("Α")),
            Expr::Word(Word::new("τρέχων")),
            Expr::Word(Word::new("ἄθροισμα")),
        ]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::BinOp { left, op, right } => {
                assert!(matches!(left.expr, AnalyzedExprKind::Variable(name) if name == "α"));
                assert!(matches!(right.expr, AnalyzedExprKind::Variable(name) if name == "τρεχων"));
                assert_eq!(op, BinaryOp::Add);
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_phrase_with_nested_block() {
        let scope = Scope::new();
        let stmt_in_block = glossa::ast::Statement::Regular {
            clauses: vec![glossa::ast::Clause {
                expressions: vec![Expr::NumberLiteral(1)],
            }],
            is_query: false,
            is_propagate: false,
        };

        let expr = Expr::Phrase(vec![Expr::Block(vec![stmt_in_block])]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::NumberLiteral(n) => assert_eq!(n, 1),
            _ => panic!("Expected NumberLiteral"),
        }
    }

    #[test]
    fn test_participle_fallback_failure() {
        let scope = Scope::new();
        let expr = Expr::Phrase(vec![Expr::Word(Word::new("τρέχων"))]);
        let result = analyze_argument_expr(&expr, &scope);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Undefined variable"));
    }

    #[test]
    fn test_participle_original_name_match() {
        let mut scope = Scope::new();
        scope.define("τρεχων", GlossaType::Number);

        let expr = Expr::Phrase(vec![Expr::Word(Word::new("τρέχων"))]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::Variable(name) => assert_eq!(name, "τρεχων"),
            _ => panic!("Expected Variable"),
        }
    }

    #[test]
    fn test_undefined_subject_failure() {
        let scope = Scope::new();
        let expr = Expr::Phrase(vec![
            Expr::Word(Word::new("Ὁ")),
            Expr::Word(Word::new("ἄνθρωπος"))
        ]);

        let result = analyze_argument_expr(&expr, &scope);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Undefined variable"));
    }

    #[test]
    fn test_phrase_complex_expressions() {
        let mut scope = Scope::new();
        scope.define("x", GlossaType::Number);
        scope.define("arr", GlossaType::List(Box::new(GlossaType::Number)));
        scope.define("y", GlossaType::Number);

        let unwrap_expr = Expr::UnaryOp {
            op: glossa::ast::UnaryOperator::Unwrap,
            operand: Box::new(Expr::Word(Word::new("x"))),
        };

        let array_expr = Expr::ArrayLiteral(vec![
            Expr::NumberLiteral(1),
            Expr::NumberLiteral(2)
        ]);

        let index_expr = Expr::IndexAccess {
            array: Box::new(Expr::Word(Word::new("arr"))),
            index: Box::new(Expr::NumberLiteral(0)),
        };

        let bool_expr = Expr::BooleanLiteral(true);
        let string_expr = Expr::StringLiteral("test".into());

        let binding_expr = Expr::Binding {
            name: Word::new("y"),
            value: Box::new(Expr::NumberLiteral(42)),
        };

        let call_expr = Expr::Call {
            verb: Word::new("λεγε"),
            arguments: vec![Expr::StringLiteral("hello".into())],
        };

        let binop_expr = Expr::BinOp {
            left: Box::new(Expr::NumberLiteral(1)),
            op: glossa::ast::BinOperator::Add,
            right: Box::new(Expr::NumberLiteral(2)),
        };

        let prop_expr = Expr::PropertyAccess {
            owner: Box::new(Expr::Word(Word::new("arr"))),
            property: Box::new(Expr::Word(Word::new("length"))),
        };

        let expr = Expr::Phrase(vec![
            unwrap_expr,
            array_expr,
            index_expr,
            bool_expr,
            binding_expr,
            call_expr,
            binop_expr,
            prop_expr,
            string_expr
        ]);

        let result = analyze_argument_expr(&expr, &scope);

        match result {
            Ok(analyzed) => {
                match analyzed.expr {
                    AnalyzedExprKind::StringLiteral(s) => assert_eq!(s, "test"),
                    _ => panic!("Expected StringLiteral, got {:?}", analyzed.expr),
                }
            },
            Err(e) => {
                panic!("Analysis failed: {:?}", e);
            }
        }
    }
}
