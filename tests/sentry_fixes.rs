use glossa::parser::parse;
use glossa::semantic::{AnalyzedExprKind, AnalyzedStatement, analyze_program};
use glossa::text::normalize_greek;

#[test]
fn test_assertion_equality_variables() {
    // Regression test for "Silent failure of equality assertion"
    // Previously, `x y ἰσοῦται` was ignored or treated as expression `x;`.
    // It should produce `AssertEq(x, y)`.
    let code = "
    ἔστω χ 5.
    ἔστω ψ 5.
    χ ψ ἰσοῦται.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    // We expect 3 statements. The last one should be AssertEq.
    assert_eq!(result.statements.len(), 3);

    let last_stmt = &result.statements[2];
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::AssertEq { left, right } = &exprs[0].expr {
            // Verify left and right are variables x and y
            match (&left.expr, &right.expr) {
                (AnalyzedExprKind::Variable(l), AnalyzedExprKind::Variable(r)) => {
                    assert!((l == "χ" && r == "ψ") || (l == "ψ" && r == "χ"));
                }
                _ => panic!("Expected variables in AssertEq"),
            }
        } else {
            panic!("Expected AssertEq expression, got {:?}", exprs[0]);
        }
    } else {
        panic!("Expected Expression statement");
    }
}

#[test]
fn test_assertion_contains_variables() {
    // Regression test for "Contains assertion defaults to 0"
    // Previously, `y in x assert` (ψ ἐν μ δεῖ) defaulted element to 0 if no literal was present.
    // Also tests smart dispatch for Collection vs Element.
    let code = "
    ἔστω ψ 5.
    μ νέον χάρτης ἔστω.
    ψ ἐν μ δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    assert_eq!(result.statements.len(), 3);

    // Stmt 3: Assert(contains)
    let last_stmt = &result.statements[2];
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall {
                receiver,
                method,
                args,
            } = &condition.expr
            {
                // Method should be contains_key (for Map) or contains
                assert!(method == "contains_key" || method == "contains");

                // Receiver should be 'μ' (Map)
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "μ");
                } else {
                    panic!("Expected receiver to be variable 'μ'");
                }

                // Arg should be 'ψ' (Element)
                // Might be wrapped in Ref
                assert_eq!(args.len(), 1);
                let arg_inner = match &args[0].expr {
                    AnalyzedExprKind::UnaryOp { op: _, operand } => &operand.expr, // Ref
                    k => k,
                };

                if let AnalyzedExprKind::Variable(name) = arg_inner {
                    assert_eq!(name, "ψ");
                } else {
                    panic!("Expected argument to be variable 'ψ'");
                }
            } else {
                panic!("Expected MethodCall");
            }
        } else {
            panic!("Expected Assert expression");
        }
    } else {
        panic!("Expected Expression statement");
    }
}

#[test]
fn test_normalization_sigma_consistency() {
    // Regression/Clarification for Sigma handling
    // Ensure normalization is consistent across case
    let upper = "Σ ";
    let norm_upper = normalize_greek(upper);

    // Previous findings showed "Σ " -> "σ " (if last_cased=false)
    // This is technically correct for isolated usage but good to verify it doesn't crash.
    assert_eq!(norm_upper, "σ ");
}

#[test]
fn test_normalization_koronis() {
    // Verify Koronis U+1FBD is preserved
    let koronis = "\u{1FBD}";
    let text = format!("κ{}αγώ", koronis);
    let normalized = normalize_greek(&text);
    assert!(normalized.contains('\u{1FBD}'));
}
