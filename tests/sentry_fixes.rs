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
    if let AnalyzedStatement::Expression(exprs) = last_stmt
        && exprs.len() == 1
        && let AnalyzedExprKind::AssertEq { left, right } = &exprs[0].expr
    {
        // Verify left and right are variables x and y
        match (&left.expr, &right.expr) {
            (AnalyzedExprKind::Variable(l), AnalyzedExprKind::Variable(r)) => {
                assert!((l == "χ" && r == "ψ") || (l == "ψ" && r == "χ"));
            }
            _ => panic!("Expected variables in AssertEq"),
        }
    } else {
        panic!("Expected AssertEq expression, got {:?}", last_stmt);
    }
}

#[test]
fn test_equality_object_variable() {
    // Coverage: Target "Object is Variable" branch in classify_equality_assertion
    // Define 'τιμή' (Nominative) but use 'τιμήν' (Accusative) in equality to target Object slot
    let code = "
    ἔστω χ 5.
    ἔστω τιμή 5.
    χ τιμήν ἰσοῦται.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    assert_eq!(result.statements.len(), 3);
    let last_stmt = &result.statements[2];
    if let AnalyzedStatement::Expression(exprs) = last_stmt
        && let AnalyzedExprKind::AssertEq { left, right } = &exprs[0].expr
    {
        match (&left.expr, &right.expr) {
            (AnalyzedExprKind::Variable(l), AnalyzedExprKind::Variable(r)) => {
                // Expect normalized lemmas: "χ" and "τιμη"
                assert!((l == "χ" && r == "τιμη") || (l == "τιμη" && r == "χ"));
            }
            _ => panic!("Expected variables in AssertEq"),
        }
    } else {
        panic!("Expected AssertEq");
    }
}

#[test]
fn test_equality_nominative_variable() {
    // Coverage: Target "Nominative is Variable" branch in classify_equality_assertion
    // Use two nominatives. "χ ψ ἰσοῦται" usually puts one in Subject, one in Nominative if word order permits?
    // Actually, assembler prefers Subject then Nominative.
    // If we have "χ ψ ἰσοῦται", 'χ' is Subject, 'ψ' is Nominative (or Object if accusative).
    // If both are nominative forms, 'ψ' goes to Nominatives list.
    // Let's ensure both are nominative. 'τιμή' and 'χ'.
    let code = "
    ἔστω χ 5.
    ἔστω τιμή 5.
    χ τιμή ἰσοῦται.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    assert_eq!(result.statements.len(), 3);
    let last_stmt = &result.statements[2];
    if let AnalyzedStatement::Expression(exprs) = last_stmt
        && let AnalyzedExprKind::AssertEq { left, right } = &exprs[0].expr
    {
        match (&left.expr, &right.expr) {
            (AnalyzedExprKind::Variable(l), AnalyzedExprKind::Variable(r)) => {
                assert!((l == "χ" && r == "τιμη") || (l == "τιμη" && r == "χ"));
            }
            _ => panic!("Expected variables in AssertEq"),
        }
    } else {
        panic!("Expected AssertEq");
    }
}

#[test]
fn test_assertion_contains_variables() {
    // Regression test for "Contains assertion defaults to 0"
    // Also tests smart dispatch: Subject=Collection (μ)
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
    if let AnalyzedStatement::Expression(exprs) = last_stmt
        && exprs.len() == 1
        && let AnalyzedExprKind::Assert { condition } = &exprs[0].expr
        && let AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } = &condition.expr
    {
        assert!(method == "contains_key" || method == "contains");
        // Receiver should be 'μ' (Map)
        if let AnalyzedExprKind::Variable(name) = &receiver.expr {
            assert_eq!(name, "μ");
        }

        // Arg should be 'ψ' (Element)
        let arg_inner = match &args[0].expr {
            AnalyzedExprKind::UnaryOp { op: _, operand } => &operand.expr, // Ref
            k => k,
        };
        if let AnalyzedExprKind::Variable(name) = arg_inner {
            assert_eq!(name, "ψ");
        }
    } else {
        panic!("Expected Assert MethodCall");
    }
}

#[test]
fn test_assertion_contains_object_collection() {
    // Coverage: Target "Object is Collection" branch in classify_assertion
    // Subject (στοιχεῖον) is Number. Object (χάρτην - acc) is Map.
    let code = "
    στοιχεῖον 5 ἔστω.
    χάρτην νέον χάρτης ἔστω.
    στοιχεῖον ἐν χάρτην δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    let last_stmt = result.statements.last().unwrap();
    if let AnalyzedStatement::Expression(exprs) = last_stmt
        && let AnalyzedExprKind::Assert { condition } = &exprs[0].expr
        && let AnalyzedExprKind::MethodCall { receiver, .. } = &condition.expr
    {
        // Receiver must be the collection 'χάρτην' -> normalized "χαρτην"
        if let AnalyzedExprKind::Variable(name) = &receiver.expr {
            assert_eq!(name, "χαρτην");
        } else {
            panic!("Smart dispatch failed to pick Object as collection");
        }
    }
}

#[test]
fn test_assertion_contains_nominative_collection() {
    // Coverage: Target "Nominative is Collection" branch in classify_assertion
    // Both Subject (στοιχεῖον) and Nominative (χάρτης) are in play.
    // Subject is Number (not collection). Nominative is Map (collection).
    let code = "
    στοιχεῖον 5 ἔστω.
    ἄλλος νέον χάρτης ἔστω.
    στοιχεῖον ἐν ἄλλος δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    let last_stmt = result.statements.last().unwrap();
    if let AnalyzedStatement::Expression(exprs) = last_stmt
        && let AnalyzedExprKind::Assert { condition } = &exprs[0].expr
        && let AnalyzedExprKind::MethodCall { receiver, .. } = &condition.expr
    {
        // Receiver must be the collection 'ἄλλος' -> normalized "αλλος"
        if let AnalyzedExprKind::Variable(name) = &receiver.expr {
            assert_eq!(name, "αλλος");
        } else {
            panic!("Smart dispatch failed to pick Nominative as collection");
        }
    }
}

#[test]
fn test_normalization_sigma_consistency() {
    // Regression/Clarification for Sigma handling
    let upper = "Σ ";
    let norm_upper = normalize_greek(upper);
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
