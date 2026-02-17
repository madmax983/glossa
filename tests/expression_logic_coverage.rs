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
        // Using "λόγον" twice to simulate two variables (Subject + Object).
        // Since both land in participles (as first element?), or maybe multiple participles?
        // Convert uses `stmt.participles.first()`. So it reuses the first participle for both Left and Right?
        // Or Assembler stores multiple participles? Yes, Vec.
        // But convert only checks `first()`. So it will be `logon + logon` (same instance effectively).

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
        // [1 λόγον +]
        // "λόγον" is Participle. 1 is Literal.
        // Convert logic checks Participle first for Left.
        // So Left = Participle (logon). Right = Literal (1).
        // Result: logon + 1. (Order swapped from input, but consistent with precedence).
        let mut scope = Scope::new();
        scope.define("λογον", GlossaType::Number);

        let expr = Expr::Phrase(vec![
            Expr::NumberLiteral(1),
            Expr::Word(Word::new("λόγον")),
            Expr::Word(Word::new("ἄθροισμα")),
        ]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::BinOp { left, op, right } => {
                // Expect swapped order due to Participle priority
                assert!(matches!(left.expr, AnalyzedExprKind::Variable(name) if name == "λογον"));
                assert!(matches!(right.expr, AnalyzedExprKind::NumberLiteral(1)));
                assert_eq!(op, BinaryOp::Add);
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_nested_phrase_expression() {
        // ((1))
        let scope = Scope::new();

        // Inner phrase (1)
        let inner = Expr::Phrase(vec![Expr::NumberLiteral(1)]);
        // Outer phrase ((1))
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
}
