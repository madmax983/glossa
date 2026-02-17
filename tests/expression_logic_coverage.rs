#[cfg(test)]
mod tests {
    use glossa::ast::{Expr, Word};
    use glossa::morphology::lexicon::BinaryOp;
    use glossa::semantic::expressions::analyze_argument_expr;
    use glossa::semantic::{AnalyzedExprKind, GlossaType, Scope};

    #[test]
    fn test_subject_operator_literal_expression() {
        // [λόγον 1 +]
        // "λόγον" is identified as a Participle in test env, so it lands in participles.
        // Convert logic prioritizes Participle as Left operand.
        // Expected: logon + 1
        let mut scope = Scope::new();
        scope.define("λογον", GlossaType::Number);

        let expr = Expr::Phrase(vec![
            Expr::Word(Word::new("λόγον")),
            Expr::NumberLiteral(1),
            Expr::Word(Word::new("ἄθροισμα")), // Add operator
        ]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::BinOp { left, op, right } => {
                assert!(matches!(left.expr, AnalyzedExprKind::Variable(name) if name == "λογον"));
                assert!(matches!(right.expr, AnalyzedExprKind::NumberLiteral(1)));
                assert_eq!(op, BinaryOp::Add);
            }
            _ => panic!("Expected BinOp, got {:?}", result.expr),
        }
    }

    #[test]
    fn test_subject_operator_object_expression() {
        // [λόγον λόγον +]
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
        // [1 μήλον +] where μήλον is strictly an Object (not Participle)
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
                // Expected: 1 + μηλον.
                // Left = 1. Right = μηλον.
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
        // ((1))
        let scope = Scope::new();
        let inner = Expr::Phrase(vec![Expr::NumberLiteral(1)]);
        let outer = Expr::Phrase(vec![inner]);

        let result = analyze_argument_expr(&outer, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::NumberLiteral(1) => {}
            _ => panic!("Expected NumberLiteral(1), got {:?}", result.expr),
        }
    }

    #[test]
    fn test_just_subject_expression() {
        // (x) where x is Subject/Participle
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
        // ( (1) 2 + )
        // Should parse as 1 + 2 = 3.
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
            // If it returns just 2 (last literal), it failed to see the binary structure
            _ => panic!("Expected BinOp, got {:?}", result.expr),
        }
    }

    #[test]
    fn test_variable_fallback_original() {
        let mut scope = Scope::new();
        scope.define("δωρα", GlossaType::Number); // Defined as plural form

        let expr = Expr::Phrase(vec![Expr::Word(Word::new("δῶρα"))]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::Variable(name) => assert_eq!(name, "δωρα"),
            _ => panic!("Expected Variable"),
        }
    }

    #[test]
    fn test_subject_nominative_expression() {
        // Subject + Nominative + Op
        let mut scope = Scope::new();
        scope.define("α", GlossaType::Number);
        scope.define("β", GlossaType::Number);

        let expr = Expr::Phrase(vec![
            Expr::Word(Word::new("Α")), // Alpha (Nom)
            Expr::Word(Word::new("Β")), // Beta (Nom)
            Expr::Word(Word::new("ἄθροισμα")),
        ]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::BinOp { left, op, right } => {
                // Expected: alpha + beta
                assert!(
                    matches!(left.expr, AnalyzedExprKind::Variable(name) if name == "α"),
                    "Left should be alpha"
                );
                assert!(
                    matches!(right.expr, AnalyzedExprKind::Variable(name) if name == "β"),
                    "Right should be beta"
                );
                assert_eq!(op, BinaryOp::Add);
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_participle_verb_lemma_fallback() {
        // Define the VERB in scope, not the participle form.
        // Participle: τρέχων (running). Lemma: τρέχω (run).
        let mut scope = Scope::new();
        scope.define("τρεχω", GlossaType::Number);

        // "τρέχων" should resolve to "τρεχω" variable via lemma fallback
        let expr = Expr::Phrase(vec![Expr::Word(Word::new("τρέχων"))]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::Variable(name) => assert_eq!(name, "τρεχω"),
            _ => panic!("Expected Variable 'τρεχω', got {:?}", result.expr),
        }
    }

    #[test]
    fn test_subject_participle_expression() {
        // Subject + Participle + Op
        // Left should be Subject. Right should fallback to Participle.
        let mut scope = Scope::new();
        scope.define("α", GlossaType::Number);
        scope.define("τρεχων", GlossaType::Number); // Define as original for simplicity

        let expr = Expr::Phrase(vec![
            Expr::Word(Word::new("Α")),      // Subject
            Expr::Word(Word::new("τρέχων")), // Participle
            Expr::Word(Word::new("ἄθροισμα")),
        ]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::BinOp { left, op, right } => {
                assert!(
                    matches!(left.expr, AnalyzedExprKind::Variable(name) if name == "α"),
                    "Left should be alpha"
                );
                assert!(
                    matches!(right.expr, AnalyzedExprKind::Variable(name) if name == "τρεχων"),
                    "Right should be trexon"
                );
                assert_eq!(op, BinaryOp::Add);
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_phrase_with_nested_block() {
        // ( {1.} )
        // Block inside Phrase.
        // Should trigger `nested_phrases` logic in `convert`.
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
            _ => panic!("Expected NumberLiteral(1), got {:?}", result.expr),
        }
    }

    #[test]
    fn test_participle_fallback_failure() {
        // (τρέχων) where "τρεχω" is NOT in scope.
        // Should fail.
        let scope = Scope::new(); // Empty scope

        let expr = Expr::Phrase(vec![Expr::Word(Word::new("τρέχων"))]);

        let result = analyze_argument_expr(&expr, &scope);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Undefined variable"));
    }

    #[test]
    fn test_phrase_complex_expressions() {
        // Create a phrase containing various expression types to exercise
        // feed_expr_to_assembler_with_context branches
        let mut scope = Scope::new();
        scope.define("x", GlossaType::Number);
        scope.define("arr", GlossaType::List(Box::new(GlossaType::Number)));

        // 1. Unary Op (Unwrap) - Postfix "!"
        let unwrap_expr = Expr::UnaryOp {
            op: glossa::ast::UnaryOperator::Unwrap,
            operand: Box::new(Expr::Word(Word::new("x"))),
        };

        // 2. Array Literal - [1, 2]
        let array_expr = Expr::ArrayLiteral(vec![
            Expr::NumberLiteral(1),
            Expr::NumberLiteral(2)
        ]);

        // 3. Index Access - arr#0
        let index_expr = Expr::IndexAccess {
            array: Box::new(Expr::Word(Word::new("arr"))),
            index: Box::new(Expr::NumberLiteral(0)),
        };

        // 4. Boolean Literal
        let bool_expr = Expr::BooleanLiteral(true);

        // 5. String Literal
        let string_expr = Expr::StringLiteral("test".into());

        // Construct phrase: (x! [1, 2] arr#0 true "test")
        let expr = Expr::Phrase(vec![
            unwrap_expr,
            array_expr,
            index_expr,
            bool_expr,
            string_expr
        ]);

        // It might error during convert if it can't build a valid expression from this soup,
        // but we are primarily testing that feed_expr_to_assembler handles these types.
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
