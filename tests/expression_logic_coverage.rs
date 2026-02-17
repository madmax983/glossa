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
        // [1 λόγον +]
        // Left priority logic might swap this if "logon" is participle.
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
                // Currently code prioritizes Participle as LEFT.
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
        // Define variable "NonNormalized" (capitalized).
        // Lookup "nonnormalized" (lemma) fails.
        // Fallback to "NonNormalized" (original).
        let mut scope = Scope::new();
        // Manually insert non-normalized key
        // Scope::define normalizes, so we must be clever.
        // Actually, Scope keys are SmolStr.
        // The issue is if the lexicon produces a lemma that is DIFFERENT from the original-normalized.
        // e.g. "dwra" (gifts) -> lemma "dwron" (gift).
        // If I define "dwra" (the plural form used in text) as variable?
        // Glossa usually binds normalized form.
        // `Esto` uses `subject.original` normalized.
        // `Assembler` stores `lemma`.
        // If I define "δῶρα" (gifts). Scope has "δωρα".
        // Assembler sees "δῶρα", lemma "δωρον".
        // `analyze_variable` looks up "δωρον". Not found.
        // Fallback: look up "δωρα" (normalized original). Found!

        scope.define("δωρα", GlossaType::Number); // Defined as plural form

        let expr = Expr::Phrase(vec![Expr::Word(Word::new("δῶρα"))]);

        let result = analyze_argument_expr(&expr, &scope).expect("Analysis failed");

        match result.expr {
            AnalyzedExprKind::Variable(name) => assert_eq!(name, "δωρα"),
            _ => panic!("Expected Variable"),
        }
    }
}
