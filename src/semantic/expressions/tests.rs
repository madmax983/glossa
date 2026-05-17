#![allow(unused_imports)]
use super::*;
use crate::ast::UnaryOperator;

#[test]
fn test_analyze_argument_expr_handles_unwrap() {
    let expr = Expr::UnaryOp {
        op: UnaryOperator::Unwrap,
        operand: Box::new(Expr::BooleanLiteral(true)),
    };

    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope).unwrap();

    match result.expr {
        AnalyzedExprKind::Unwrap(_) => {
            // Success - correctly identified as Unwrap
        }
        _ => panic!("Expected Unwrap, got {:?}", result),
    }
}

#[test]
fn test_build_expressions_insufficient_literals() {
    // Case: 1 + 2 +
    // Literals: [1, 2]
    // Operators: [Add, Add]
    // Expected: Should return Error due to insufficient literals

    let literals = vec![Literal::Number(1), Literal::Number(2)];
    let operators = vec![
        crate::morphology::lexicon::BinaryOp::Add,
        crate::morphology::lexicon::BinaryOp::Add,
    ];

    let result = build_expressions_from_literals_and_ops(&literals, &operators);

    assert!(
        result.is_err(),
        "Expected error for dangling operator, got {:?}",
        result
    );

    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Insufficient literals"),
        "Unexpected error message: {}",
        err
    );
}

#[test]
fn test_analyze_argument_expr_handles_array() {
    let expr = Expr::ArrayLiteral(vec![Expr::NumberLiteral(1), Expr::NumberLiteral(2)]);
    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope).unwrap();

    match result.expr {
        AnalyzedExprKind::ArrayLiteral(elements) => {
            assert_eq!(elements.len(), 2);
            assert!(matches!(
                elements[0].expr,
                AnalyzedExprKind::NumberLiteral(1)
            ));
        }
        _ => panic!("Expected ArrayLiteral"),
    }
}

#[test]
fn test_analyze_argument_expr_handles_index_access() {
    let expr = Expr::IndexAccess {
        array: Box::new(Expr::ArrayLiteral(vec![])),
        index: Box::new(Expr::NumberLiteral(0)),
    };
    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope).unwrap();

    match result.expr {
        AnalyzedExprKind::IndexAccess { array: _, index } => {
            assert!(matches!(index.expr, AnalyzedExprKind::NumberLiteral(0)));
        }
        _ => panic!("Expected IndexAccess"),
    }
}

#[test]
fn test_analyze_argument_expr_handles_binop() {
    let scope = Scope::new();
    let ops = vec![
        (
            crate::ast::BinOperator::Add,
            crate::morphology::lexicon::BinaryOp::Add,
        ),
        (
            crate::ast::BinOperator::Sub,
            crate::morphology::lexicon::BinaryOp::Sub,
        ),
        (
            crate::ast::BinOperator::Mul,
            crate::morphology::lexicon::BinaryOp::Mul,
        ),
        (
            crate::ast::BinOperator::Div,
            crate::morphology::lexicon::BinaryOp::Div,
        ),
        (
            crate::ast::BinOperator::Mod,
            crate::morphology::lexicon::BinaryOp::Mod,
        ),
        (
            crate::ast::BinOperator::Eq,
            crate::morphology::lexicon::BinaryOp::Eq,
        ),
        (
            crate::ast::BinOperator::Ne,
            crate::morphology::lexicon::BinaryOp::Ne,
        ),
        (
            crate::ast::BinOperator::Lt,
            crate::morphology::lexicon::BinaryOp::Lt,
        ),
        (
            crate::ast::BinOperator::Le,
            crate::morphology::lexicon::BinaryOp::Le,
        ),
        (
            crate::ast::BinOperator::Gt,
            crate::morphology::lexicon::BinaryOp::Gt,
        ),
        (
            crate::ast::BinOperator::Ge,
            crate::morphology::lexicon::BinaryOp::Ge,
        ),
        (
            crate::ast::BinOperator::And,
            crate::morphology::lexicon::BinaryOp::And,
        ),
        (
            crate::ast::BinOperator::Or,
            crate::morphology::lexicon::BinaryOp::Or,
        ),
    ];

    for (ast_op, expected_sem_op) in ops {
        let expr = Expr::BinOp {
            left: Box::new(Expr::NumberLiteral(1)),
            op: ast_op,
            right: Box::new(Expr::NumberLiteral(2)),
        };
        let result = analyze_argument_expr(&expr, &scope).unwrap();

        match result.expr {
            AnalyzedExprKind::BinOp { op, .. } => {
                assert_eq!(
                    op, expected_sem_op,
                    "Mismatch for AST operator {:?}",
                    ast_op
                );
            }
            _ => panic!("Expected BinOp"),
        }
    }
}

#[test]
fn test_analyze_argument_expr_handles_property_access() {
    let expr = Expr::PropertyAccess {
        owner: Box::new(Expr::Word(crate::ast::Word::new("x"))),
        property: Box::new(Expr::Word(crate::ast::Word::new("y"))),
    };
    let mut scope = Scope::new();
    scope.define("x", GlossaType::Unknown);
    let result = analyze_argument_expr(&expr, &scope).unwrap();

    match result.expr {
        AnalyzedExprKind::PropertyAccess { property, .. } => {
            assert_eq!(property, "y");
        }
        _ => panic!("Expected PropertyAccess"),
    }
}

#[test]
fn test_analyze_argument_expr_errors_on_invalid_property() {
    let expr = Expr::PropertyAccess {
        owner: Box::new(Expr::Word(crate::ast::Word::new("x"))),
        property: Box::new(Expr::NumberLiteral(1)),
    };
    let mut scope = Scope::new();
    scope.define("x", GlossaType::Unknown);
    let result = analyze_argument_expr(&expr, &scope);

    assert!(result.is_err());
}

#[test]
fn test_analyze_argument_expr_errors_on_empty_phrase() {
    let expr = Expr::Phrase(vec![]);
    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope);

    assert!(result.is_err());
}

#[test]
fn test_analyze_argument_expr_propagates_error_in_array() {
    // Array with an invalid element (empty phrase)
    let expr = Expr::ArrayLiteral(vec![Expr::NumberLiteral(1), Expr::Phrase(vec![])]);
    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope);

    assert!(result.is_err());
}

#[test]
fn test_analyze_argument_expr_propagates_error_in_index_access() {
    // Index access with invalid array
    let expr = Expr::IndexAccess {
        array: Box::new(Expr::Phrase(vec![])),
        index: Box::new(Expr::NumberLiteral(0)),
    };
    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope);

    assert!(result.is_err());

    // Index access with invalid index
    let expr2 = Expr::IndexAccess {
        array: Box::new(Expr::ArrayLiteral(vec![])),
        index: Box::new(Expr::Phrase(vec![])),
    };
    let result2 = analyze_argument_expr(&expr2, &scope);
    assert!(result2.is_err());
}

#[test]
fn test_analyze_argument_expr_propagates_error_in_binop() {
    // BinOp with invalid left
    let expr = Expr::BinOp {
        left: Box::new(Expr::Phrase(vec![])),
        op: crate::ast::BinOperator::Add,
        right: Box::new(Expr::NumberLiteral(1)),
    };
    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope);
    assert!(result.is_err());

    // BinOp with invalid right
    let expr2 = Expr::BinOp {
        left: Box::new(Expr::NumberLiteral(1)),
        op: crate::ast::BinOperator::Add,
        right: Box::new(Expr::Phrase(vec![])),
    };
    let result2 = analyze_argument_expr(&expr2, &scope);
    assert!(result2.is_err());
}

#[test]
fn test_analyze_argument_expr_propagates_error_in_unary_op() {
    // UnaryOp with invalid operand
    let expr = Expr::UnaryOp {
        op: crate::ast::UnaryOperator::Unwrap,
        operand: Box::new(Expr::Phrase(vec![])),
    };
    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope);
    assert!(result.is_err());
}

#[test]
fn test_analyze_argument_expr_propagates_error_in_property_owner() {
    // Property access with invalid owner
    let expr = Expr::PropertyAccess {
        owner: Box::new(Expr::Phrase(vec![])),
        property: Box::new(Expr::Word(crate::ast::Word::new("y"))),
    };
    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope);
    assert!(result.is_err());
}

#[test]
fn test_analyze_argument_expr_handles_phrase_recursion() {
    // Phrase that is not a function call -> recursive analysis of first term
    // "((1))" -> Phrase(vec![Phrase(vec![Number(1)])])
    let inner = Expr::Phrase(vec![Expr::NumberLiteral(1)]);
    let outer = Expr::Phrase(vec![inner]);
    let scope = Scope::new();
    let result = analyze_argument_expr(&outer, &scope).unwrap();

    match result.expr {
        AnalyzedExprKind::NumberLiteral(n) => assert_eq!(n, 1),
        _ => panic!("Expected NumberLiteral"),
    }
}

#[test]
fn test_analyze_argument_expr_handles_function_call() {
    // Mock a function in scope
    let mut scope = Scope::new();
    scope.define_function(
        "add",
        vec![GlossaType::Number, GlossaType::Number],
        Some(GlossaType::Number),
    );

    // "add 1 2"
    let expr = Expr::Phrase(vec![
        Expr::Word(crate::ast::Word::new("add")),
        Expr::NumberLiteral(1),
        Expr::NumberLiteral(2),
    ]);

    let result = analyze_argument_expr(&expr, &scope).unwrap();

    match result.expr {
        AnalyzedExprKind::FunctionCall { func, args } => {
            assert_eq!(func, "add");
            assert_eq!(args.len(), 2);
            assert!(matches!(args[0].expr, AnalyzedExprKind::NumberLiteral(1)));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_analyze_argument_expr_propagates_error_in_function_args() {
    let mut scope = Scope::new();
    scope.define_function("add", vec![GlossaType::Number], Some(GlossaType::Number));

    // "add (error)"
    let expr = Expr::Phrase(vec![
        Expr::Word(crate::ast::Word::new("add")),
        Expr::Phrase(vec![]), // Invalid arg
    ]);

    let result = analyze_argument_expr(&expr, &scope);
    assert!(result.is_err());
}

#[test]
fn test_analyze_argument_expr_handles_block() {
    // Block containing a statement with a clause with an expression
    // { 1. }
    let stmt = crate::ast::Statement::Regular {
        clauses: vec![crate::ast::Clause {
            expressions: vec![Expr::NumberLiteral(1)],
        }],
        is_query: false,
        is_propagate: false,
    };
    let expr = Expr::Block(vec![stmt]);
    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope).unwrap();

    match result.expr {
        AnalyzedExprKind::NumberLiteral(n) => assert_eq!(n, 1),
        _ => panic!("Expected NumberLiteral"),
    }
}

#[test]
fn test_analyze_argument_expr_errors_on_empty_block() {
    let expr = Expr::Block(vec![]);
    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope);
    assert!(result.is_err());
}

#[test]
fn test_analyze_argument_expr_errors_on_invalid_block_structure() {
    // Block with a statement that has no clauses (should trigger error)
    let stmt = crate::ast::Statement::Regular {
        clauses: vec![],
        is_query: false,
        is_propagate: false,
    };
    let expr = Expr::Block(vec![stmt]);
    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope);
    assert!(result.is_err());
}

#[test]
fn test_analyze_argument_expr_errors_on_block_with_empty_clause() {
    // Block with a statement that has a clause with no expressions
    let stmt = crate::ast::Statement::Regular {
        clauses: vec![crate::ast::Clause {
            expressions: vec![],
        }],
        is_query: false,
        is_propagate: false,
    };
    let expr = Expr::Block(vec![stmt]);
    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope);
    assert!(result.is_err());
}

#[test]
fn test_vso_ambiguity_resolution() {
    use crate::ast::Word;
    use crate::semantic::{Assembler, DisambiguationContext};

    // Test sentence: λέγω τὸ πρῶτον
    // "I say" (1st Person) "the first" (Neuter Nom/Acc 3rd Person?)
    // Actually "πρῶτον" is an adjective but used as noun here.
    // Or better: Use "λέγω τὸ ὄνομα" (I say the name).
    // "λέγω" (1st Person) "ὄνομα" (Neuter Nom/Acc 3rd Person).
    // Should parse as: Verb(I say) + Object(name)
    // NOT: Subject(name) + Verb(I say) -> Agreement Error

    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();

    // 1. Feed "λέγω" (I say)
    let verb = Expr::Word(Word::new("λέγω"));
    feed_expr_to_assembler_with_context(&mut asm, &verb, &mut ctx).unwrap();

    // 2. Feed "τό" (the)
    let article = Expr::Word(Word::new("τό"));
    feed_expr_to_assembler_with_context(&mut asm, &article, &mut ctx).unwrap();

    // 3. Feed "ὄνομα" (name)
    let noun = Expr::Word(Word::new("ὄνομα"));
    feed_expr_to_assembler_with_context(&mut asm, &noun, &mut ctx).unwrap();

    // 4. Finalize
    let stmt = asm.finalize().unwrap();

    // Verify
    assert!(stmt.verb.is_some(), "Should have a verb");
    assert_eq!(stmt.verb.as_ref().unwrap().original, "λέγω");

    assert!(stmt.object.is_some(), "Should have an object");
    assert_eq!(stmt.object.as_ref().unwrap().original, "ὄνομα");

    // Should NOT have a subject (implicit "I")
    assert!(
        stmt.subject.is_none(),
        "Should NOT have a subject (found: {:?})",
        stmt.subject
    );
}

#[test]
fn test_backtracking_failure_propagates_error() {
    use crate::ast::Word;
    use crate::semantic::{Assembler, DisambiguationContext};

    // Test sentence: ἐγὼ τρέχει
    // "I" (Subj 1st) "runs" (Verb 3rd) -> Agreement Error
    // This should fail for ALL backtracking candidates of "τρέχει".
    // We verify that the error is propagated.

    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();

    // 1. Feed "ἐγώ" (I)
    let subj = Expr::Word(Word::new("ἐγώ"));
    feed_expr_to_assembler_with_context(&mut asm, &subj, &mut ctx).unwrap();

    // 2. Feed "τρέχει" (runs - 3rd person)
    let verb = Expr::Word(Word::new("τρέχει"));
    let result = feed_expr_to_assembler_with_context(&mut asm, &verb, &mut ctx);

    assert!(
        result.is_err(),
        "Backtracking should fail when no candidates match agreement"
    );
    let err = result.unwrap_err();
    // The error message comes from GlossaError::semantic wrapping AssemblyError
    // AssemblyError::SubjectVerbDisagreement -> Localized "Ἀσυμφωνία" (Disagreement)
    assert!(
        err.to_string().contains("Ἀσυμφωνία"),
        "Error should be SubjectVerbDisagreement (Ἀσυμφωνία), got: {}",
        err
    );
}

#[test]
fn test_phrase_errors_on_multiple_terms() {
    let expr = Expr::Phrase(vec![Expr::NumberLiteral(1), Expr::NumberLiteral(2)]);
    let scope = Scope::new();
    let result = analyze_argument_expr(&expr, &scope);

    // This test should fail currently because the code returns Ok(1)
    assert!(
        result.is_err(),
        "Should error on multiple terms in non-function phrase, but got: {:?}",
        result
    );
}

#[test]
fn test_build_expressions_preserves_literals() {
    let literals = vec![Literal::Number(1), Literal::Number(2), Literal::Number(3)];
    let operators = vec![crate::morphology::lexicon::BinaryOp::Add];

    let exprs = build_expressions_from_literals_and_ops(&literals, &operators).unwrap();

    assert_eq!(
        exprs.len(),
        2,
        "Should return 2 expressions, got: {:?}",
        exprs
    );

    if let AnalyzedExprKind::NumberLiteral(n) = &exprs[1].expr {
        assert_eq!(*n, 3);
    } else {
        panic!("Second expression should be NumberLiteral(3)");
    }
}

#[test]
fn test_dropped_operator_insufficient_literals() {
    // Case: 1 + 2 +
    // Literals: [1, 2]
    // Operators: [Add, Add]
    // Expected: Should return Error due to insufficient literals

    let literals = vec![Literal::Number(1), Literal::Number(2)];
    let operators = vec![
        crate::morphology::lexicon::BinaryOp::Add,
        crate::morphology::lexicon::BinaryOp::Add,
    ];

    let result = build_expressions_from_literals_and_ops(&literals, &operators);

    assert!(
        result.is_err(),
        "Expected error for dangling operator, got {:?}",
        result
    );

    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Insufficient literals"),
        "Unexpected error message: {}",
        err
    );
}

#[test]
fn test_recursion_limit_expression_analysis() {
    // Construct a deeply nested Expr structure
    // Expr::Phrase -> Expr::Phrase -> ... (51 times)
    let mut deep_expr = Expr::NumberLiteral(1);
    for _ in 0..52 {
        deep_expr = Expr::Phrase(vec![deep_expr]);
    }

    let scope = Scope::new();
    let result = analyze_argument_expr(&deep_expr, &scope);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Recursion limit exceeded"));
}

#[test]
fn test_dropped_operator() {
    // Case: 1 + 2 +
    // Literals: [1, 2]
    // Operators: [Add, Add]
    // Expected: Should return Error due to insufficient literals

    let literals = vec![Literal::Number(1), Literal::Number(2)];
    let operators = vec![
        crate::morphology::lexicon::BinaryOp::Add,
        crate::morphology::lexicon::BinaryOp::Add,
    ];

    let result = build_expressions_from_literals_and_ops(&literals, &operators);

    assert!(
        result.is_err(),
        "Expected error for dangling operator, got {:?}",
        result
    );

    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Insufficient literals"),
        "Unexpected error message: {}",
        err
    );
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    use crate::morphology::lexicon::BinaryOp;

    #[test]
    fn test_dropped_operator_regression() {
        // Case: 1 + 2 +
        // Literals: [1, 2]
        // Operators: [Add, Add]
        // Expected: Should return Error due to insufficient literals

        let literals = vec![Literal::Number(1), Literal::Number(2)];
        let operators = vec![BinaryOp::Add, BinaryOp::Add];

        let result = build_expressions_from_literals_and_ops(&literals, &operators);

        assert!(
            result.is_err(),
            "Expected error for dangling operator, got {:?}",
            result
        );

        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Insufficient literals"),
            "Unexpected error message: {}",
            err
        );
    }
}
