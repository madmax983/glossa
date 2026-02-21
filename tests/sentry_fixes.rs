#![allow(clippy::collapsible_if)]

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
        if exprs.len() == 1 {
            if let AnalyzedExprKind::AssertEq { left, right } = &exprs[0].expr {
                // Verify left and right are variables x and y
                match (&left.expr, &right.expr) {
                    (AnalyzedExprKind::Variable(l), AnalyzedExprKind::Variable(r)) => {
                        assert!((l == "χ" && r == "ψ") || (l == "ψ" && r == "χ"));
                    }
                    _ => panic!("Expected variables in AssertEq"),
                }
                return;
            }
        }
    }
    panic!("Expected AssertEq expression, got {:?}", last_stmt);
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
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::AssertEq { left, right } = &exprs[0].expr {
            match (&left.expr, &right.expr) {
                (AnalyzedExprKind::Variable(l), AnalyzedExprKind::Variable(r)) => {
                    // Expect normalized lemmas: "χ" and "τιμη"
                    assert!((l == "χ" && r == "τιμη") || (l == "τιμη" && r == "χ"));
                }
                _ => panic!("Expected variables in AssertEq"),
            }
            return;
        }
    }
    panic!("Expected AssertEq");
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
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::AssertEq { left, right } = &exprs[0].expr {
            match (&left.expr, &right.expr) {
                (AnalyzedExprKind::Variable(l), AnalyzedExprKind::Variable(r)) => {
                    assert!((l == "χ" && r == "τιμη") || (l == "τιμη" && r == "χ"));
                }
                _ => panic!("Expected variables in AssertEq"),
            }
            return;
        }
    }
    panic!("Expected AssertEq");
}

#[test]
fn test_equality_literal() {
    // Coverage: Target "Literal" branch in classify_equality_assertion
    let code = "
    ἔστω χ 5.
    χ 5 ἰσοῦται.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    assert_eq!(result.statements.len(), 2);
    let last_stmt = &result.statements[1];
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::AssertEq { left, right } = &exprs[0].expr {
            if let AnalyzedExprKind::Variable(l) = &left.expr {
                assert_eq!(l, "χ");
            } else {
                panic!("Left should be variable");
            }

            if let AnalyzedExprKind::NumberLiteral(r) = &right.expr {
                assert_eq!(*r, 5);
            } else {
                panic!("Right should be number");
            }
            return;
        }
    }
    panic!("Expected AssertEq");
}

#[test]
fn test_assertion_equality_literal_no_variables() {
    // Coverage: Target branch where literal exists but no variables are found (no subject/object/nominative).
    // "5 ἰσοῦται" (5 equals...) -> Only literal 5. No Subject.
    // This should fail to produce AssertEq and fallback to expression.
    let code = "
    5 ἰσοῦται.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    assert_eq!(result.statements.len(), 1);
    if let AnalyzedStatement::Expression(exprs) = &result.statements[0] {
        // Should contain just the literal 5
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::NumberLiteral(n) = &exprs[0].expr {
            assert_eq!(*n, 5);
        } else {
            panic!("Expected NumberLiteral(5)");
        }

        // Should NOT be AssertEq
        if matches!(exprs[0].expr, AnalyzedExprKind::AssertEq { .. }) {
            panic!("Should not generate AssertEq without second operand");
        }
    }
}

#[test]
fn test_assertion_equality_no_variables() {
    // Coverage: Target the `else` branch in `classify_equality_assertion`
    // where no literal or object/nominative variable is found.
    // This should result in NO AssertEq statement being generated (None),
    // and thus NO statement in the final output (or just the subject expression if fallback kicks in).

    // "x equals" - missing second operand
    let code = "
    ἔστω χ 5.
    χ ἰσοῦται.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    // Actually, it seems we only get 2 statements if classify_expression kicks in.
    // Statement 1: Binding x
    // Statement 2: Expression(x)

    assert_eq!(result.statements.len(), 2);
    let last_stmt = &result.statements[1];

    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        // Should NOT be AssertEq
        if let AnalyzedExprKind::AssertEq { .. } = &exprs[0].expr {
            panic!("Should not have generated AssertEq with missing operand");
        }
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
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if exprs.len() == 1 {
            if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
                if let AnalyzedExprKind::MethodCall {
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
                    return;
                }
            }
        }
    }
    panic!("Expected Assert MethodCall");
}

#[test]
fn test_assertion_contains_subject_collection_object_element() {
    // Coverage: Target `is_subj_collection` -> `object` branch
    // Subject: Collection
    // Object: Element
    // "μ(Map) τὸν ψ(Element, Acc) δεῖ" (Map needs/asserts the psi)
    // Preposition 'ἐν' might be needed for proper classification as assertion?
    // The code says `if asm_stmt.has_containment_preposition`.
    // So we need 'ἐν'.
    // "τὸν ψ ἐν μ δεῖ."
    let code = "
    ἔστω ψ 5.
    μ νέον χάρτης ἔστω.
    τὸν ψ ἐν μ δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    let last_stmt = result.statements.last().unwrap();
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall { receiver, args, .. } = &condition.expr {
                // Receiver: μ
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "μ");
                }
                // Arg: ψ
                let arg_inner = match &args[0].expr {
                    AnalyzedExprKind::UnaryOp { op: _, operand } => &operand.expr,
                    k => k,
                };
                if let AnalyzedExprKind::Variable(name) = arg_inner {
                    assert_eq!(name, "ψ");
                }
                return;
            }
        }
    }
    panic!("Expected Assert MethodCall");
}

#[test]
fn test_assertion_contains_subject_collection_literal() {
    // Coverage: Target `is_subj_collection` -> `literal` branch
    // Subject: Collection
    // Literal: 5
    let code = "
    μ νέον χάρτης ἔστω.
    5 ἐν μ δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    let last_stmt = result.statements.last().unwrap();
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall { receiver, args, .. } = &condition.expr {
                // Receiver: μ
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "μ");
                }
                // Arg: 5 (Literal)
                // May be wrapped in Ref if it's not string, but NumberLiteral is handled by generic wrapper.
                let arg_inner = match &args[0].expr {
                    AnalyzedExprKind::UnaryOp { op: _, operand } => &operand.expr,
                    k => k,
                };
                if let AnalyzedExprKind::NumberLiteral(n) = arg_inner {
                    assert_eq!(*n, 5);
                } else {
                    panic!("Expected NumberLiteral(5)");
                }
                return;
            }
        }
    }
    panic!("Expected Assert MethodCall");
}

#[test]
fn test_assertion_contains_subject_collection_default() {
    // Coverage: Target `is_subj_collection` -> `else` (default 0) branch
    // Subject: Collection
    // No element specified
    let code = "
    μ νέον χάρτης ἔστω.
    ἐν μ δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    let last_stmt = result.statements.last().unwrap();
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall { receiver, args, .. } = &condition.expr {
                // Receiver: μ
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "μ");
                }
                // Arg: 0 (Default)
                let arg_inner = match &args[0].expr {
                    AnalyzedExprKind::UnaryOp { op: _, operand } => &operand.expr,
                    k => k,
                };
                if let AnalyzedExprKind::NumberLiteral(n) = arg_inner {
                    assert_eq!(*n, 0);
                } else {
                    panic!("Expected default 0");
                }
                return;
            }
        }
    }
    panic!("Expected Assert MethodCall");
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
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall { receiver, .. } = &condition.expr {
                // Receiver must be the collection 'χάρτην' -> normalized "χαρτην"
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "χαρτην");
                } else {
                    panic!("Smart dispatch failed to pick Object as collection");
                }
                return;
            }
        }
    }
    panic!("Expected Assert MethodCall");
}

#[test]
fn test_assertion_contains_nominative_collection() {
    // Coverage: Target "Nominative is Collection" branch in classify_assertion
    // Both Subject (στοιχεῖον) and Nominative (χάρτης) are in play.
    // Subject is Number (not collection). Nominative is Map (collection).
    // We add `ἐν αγνωστος` (in unknown) to consume the Object slot and force Nominative check.
    let code = "
    στοιχεῖον 5 ἔστω.
    ἄλλος νέον χάρτης ἔστω.
    στοιχεῖον ἄλλος ἐν αγνωστος δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    let last_stmt = result.statements.last().unwrap();
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall { receiver, .. } = &condition.expr {
                // Receiver must be the collection 'ἄλλος' -> normalized "αλλος"
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "αλλος");
                } else {
                    panic!("Smart dispatch failed to pick Nominative as collection");
                }
                return;
            }
        }
    }
    panic!("Expected Assert MethodCall");
}

#[test]
fn test_assertion_contains_fallback_variable() {
    // Coverage: Target the Fallback branch in classify_assertion (when no collection type is found)
    // AND test that variable resolution works in fallback (the bug fix).
    // Use types that are NOT collections (Numbers).
    // "ψ in χ assert" -> "ψ ἐν χ δεῖ"
    // Subject: ψ (Nominative)
    // Nominatives: [χ] (Nominative)
    // Since 'ψ' is Number (not Map/List/Set), smart dispatch will fail.
    // Fallback should pick 'ψ' (Subject) as receiver.
    // And 'χ' (Nominative) as element.
    let code = "
    ἔστω χ 5.
    ἔστω ψ 5.
    ψ ἐν χ δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    let last_stmt = result.statements.last().unwrap();
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall { receiver, args, .. } = &condition.expr {
                // Receiver should be 'ψ' (fallback to subject)
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "ψ");
                } else {
                    panic!(
                        "Expected receiver to be 'ψ' in fallback, got {:?}",
                        receiver
                    );
                }

                // Arg should be 'x' (element from nominatives)
                let arg_inner = match &args[0].expr {
                    AnalyzedExprKind::UnaryOp { op: _, operand } => &operand.expr, // Ref
                    k => k,
                };
                if let AnalyzedExprKind::Variable(name) = arg_inner {
                    assert_eq!(name, "χ");
                } else {
                    panic!(
                        "Fallback failed to resolve variable argument 'χ', got {:?}",
                        arg_inner
                    );
                }
                return;
            }
        }
    }
    panic!("Expected Assert MethodCall");
}

#[test]
fn test_assertion_contains_fallback_object_element() {
    // Coverage: Target Fallback -> Object Element branch
    // Subject: χ (Number) -> Not Collection, triggers fallback.
    // Object: τὸν ψ (Number) -> Should be picked as element.
    let code = "
    ἔστω χ 5.
    ἔστω ψ 5.
    χ ἐν τὸν ψ δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    let last_stmt = result.statements.last().unwrap();
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall { receiver, args, .. } = &condition.expr {
                // Receiver: χ (Subject)
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "χ");
                }
                // Element: ψ (Object)
                let arg_inner = match &args[0].expr {
                    AnalyzedExprKind::UnaryOp { op: _, operand } => &operand.expr,
                    k => k,
                };
                if let AnalyzedExprKind::Variable(name) = arg_inner {
                    assert_eq!(name, "ψ");
                }
                return;
            }
        }
    }
    panic!("Expected Assert MethodCall");
}

#[test]
fn test_assertion_contains_fallback_literal() {
    // Coverage: Target Fallback -> Literal Element branch
    // Subject: χ
    // Literal: 5
    let code = "
    ἔστω χ 5.
    5 ἐν χ δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    let last_stmt = result.statements.last().unwrap();
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall { receiver, args, .. } = &condition.expr {
                // Receiver: χ
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "χ");
                }
                // Element: 5 (Literal)
                let arg_inner = match &args[0].expr {
                    AnalyzedExprKind::UnaryOp { op: _, operand } => &operand.expr,
                    k => k,
                };
                if let AnalyzedExprKind::NumberLiteral(n) = arg_inner {
                    assert_eq!(*n, 5);
                } else {
                    panic!("Expected NumberLiteral(5)");
                }
                return;
            }
        }
    }
    panic!("Expected Assert MethodCall");
}

#[test]
fn test_assertion_contains_fallback_default() {
    // Coverage: Target Fallback -> Default (0)
    // No variable element provided.
    // "In x assert" -> "ἐν χ δεῖ"
    let code = "
    ἔστω χ 5.
    ἐν χ δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    let last_stmt = result.statements.last().unwrap();
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall { receiver, args, .. } = &condition.expr {
                // Receiver: χ
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "χ");
                }
                // Element: 0 (Default)
                let arg_inner = match &args[0].expr {
                    AnalyzedExprKind::UnaryOp { op: _, operand } => &operand.expr,
                    k => k,
                };
                if let AnalyzedExprKind::NumberLiteral(n) = arg_inner {
                    assert_eq!(*n, 0);
                } else {
                    panic!("Expected default 0");
                }
                return;
            }
        }
    }
    panic!("Expected Assert MethodCall");
}

#[test]
fn test_assertion_contains_subject_collection_nominative_element() {
    // Coverage: Subject is Collection, Element is Nominative (second variable, not object)
    // We force this by providing an "unknown" object via `ἐν αγνωστος`.
    // This makes `obj_var` None, forcing the code to look at Nominatives.
    let code = "
    μ νέον χάρτης ἔστω.
    ἔστω ψ 5.
    μ ψ ἐν αγνωστος δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    let last_stmt = result.statements.last().unwrap();
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall { receiver, args, .. } = &condition.expr {
                // Receiver: μ (Subject Collection)
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "μ");
                }
                // Element: ψ (Nominative)
                let arg_inner = match &args[0].expr {
                    AnalyzedExprKind::UnaryOp { op: _, operand } => &operand.expr,
                    k => k,
                };
                if let AnalyzedExprKind::Variable(name) = arg_inner {
                    assert_eq!(name, "ψ");
                }
                return;
            }
        }
    }
    panic!("Expected Assert MethodCall for Nominative Element");
}

#[test]
fn test_assertion_contains_fallback_nominative_element() {
    // Coverage: Fallback (Subject !Collection), Element is Nominative
    // We force this by providing an "unknown" object via `ἐν αγνωστος`.
    let code = "
    ἔστω χ 5.
    ἔστω ψ 5.
    χ ψ ἐν αγνωστος δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    let last_stmt = result.statements.last().unwrap();
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall { receiver, args, .. } = &condition.expr {
                // Receiver: χ (Subject - Fallback)
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "χ");
                }
                // Element: ψ (Nominative - Fallback)
                let arg_inner = match &args[0].expr {
                    AnalyzedExprKind::UnaryOp { op: _, operand } => &operand.expr,
                    k => k,
                };
                if let AnalyzedExprKind::Variable(name) = arg_inner {
                    assert_eq!(name, "ψ");
                }
                return;
            }
        }
    }
    panic!("Expected Assert MethodCall for Fallback Nominative");
}

#[test]
fn test_assertion_contains_nominative_collection_via_original_lookup() {
    // Coverage: Nominative is Collection, found via `original` lookup
    // Variable `σώματα` (Neuter Plural). Lemma `σῶμα`.
    // Scope has `σωματα`. Lookup `σωμα` fails. Lookup `σωματα` succeeds.
    // Use `σώματα` with `ἔστω` (Attic syntax allowed for Neuter Plural).
    let code = "
    σώματα νέον χάρτης ἔστω.
    στοιχεῖον 5 ἔστω.
    στοιχεῖον σώματα ἐν αγνωστος δεῖ.
    ";

    let parsed = parse(code).expect("Parse failed");
    let result = analyze_program(&parsed).expect("Analysis failed");

    let last_stmt = result.statements.last().unwrap();
    if let AnalyzedStatement::Expression(exprs) = last_stmt {
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall { receiver, .. } = &condition.expr {
                // Receiver: σώματα (via original lookup)
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "σωματα");
                }
                return;
            }
        }
    }
    panic!("Expected Assert MethodCall via Original Nominative Lookup");
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
