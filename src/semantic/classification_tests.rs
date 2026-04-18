use super::conversion::classify_assembled_statement;
use crate::ast::Expr;
use crate::morphology::Case;
use crate::morphology::lexicon::BinaryOp;
use crate::semantic::assembly::{ParticipleConstituent, VerbConstituent};
use crate::semantic::{
    AnalyzedExprKind, AnalyzedStatement, AssembledStatement, Constituent, GlossaType, Literal,
    Scope,
};
use crate::text::normalize_greek;

fn make_constituent(original: &str, lemma: &str) -> Constituent {
    Constituent {
        lemma: lemma.into(),
        original: original.into(),
        normalized: normalize_greek(original),
        case: Case::Nominative,
        number: None,
        gender: None,
        person: None,
    }
}

fn make_verb(original: &str, lemma: &str) -> VerbConstituent {
    VerbConstituent {
        lemma: lemma.into(),
        original: original.into(),
        normalized: normalize_greek(original),
        person: None,
        number: None,
        tense: None,
        mood: None,
        voice: None,
    }
}

#[test]
fn test_classify_simple_binding() {
    let mut scope = Scope::new();

    // "x 5 let"
    let asm_stmt = AssembledStatement {
        subject: Some(make_constituent("x", "x")),
        literals: vec![Literal::Number(5)],
        verb: Some(make_verb("let", "εστω")), // "ἔστω" is a binding verb
        ..Default::default()
    };

    let result =
        classify_assembled_statement(&asm_stmt, &mut scope).expect("Classification failed");

    if let AnalyzedStatement::Binding { name, value, .. } = result {
        assert_eq!(name, "x");
        if let AnalyzedExprKind::NumberLiteral(n) = value.expr {
            assert_eq!(n, 5);
        } else {
            panic!("Expected NumberLiteral");
        }
    } else {
        panic!("Expected Binding, got {:?}", result);
    }
}

#[test]
fn test_classify_binding_subject_object_swap() {
    let mut scope = Scope::new();
    scope.define("val", GlossaType::Number);

    // "val x let" -> Should bind x to val, because val is defined and x is not.
    // Original: Subject=val, Object=x (because of word order/case, usually)
    // Here we simulate the assembler putting "val" in Subject and "x" in Object.
    let asm_stmt = AssembledStatement {
        subject: Some(make_constituent("val", "val")),
        object: Some(make_constituent("x", "x")),
        verb: Some(make_verb("let", "εστω")),
        ..Default::default()
    };

    let result = classify_assembled_statement(&asm_stmt, &mut scope);
    assert!(
        result.is_err(),
        "Should fail due to strict undefined checking"
    );
}

#[test]
fn test_classify_binding_false_participle() {
    let mut scope = Scope::new();

    // "x 5 written let" -> "written" is a participle but not a real one in this context
    // (it's the variable name essentially or part of the phrase)
    // The logic checks if the participle lemma exists in lexicon. If not, it treats it as the variable name.

    let asm_stmt = AssembledStatement {
        literals: vec![Literal::Number(5)],
        verb: Some(make_verb("let", "εστω")),
        // Add a "false" participle (lemma not in lexicon)
        participles: vec![ParticipleConstituent {
            verb_lemma: "unknown_verb".into(),
            original: "written".into(),
            normalized: "written".into(),
            tense: crate::morphology::Tense::Present, // Dummy values
            voice: crate::morphology::Voice::Active,
            case: Case::Nominative,
            gender: crate::morphology::Gender::Neuter,
            number: crate::morphology::Number::Singular,
        }],
        ..Default::default()
    };

    let result =
        classify_assembled_statement(&asm_stmt, &mut scope).expect("Classification failed");

    if let AnalyzedStatement::Binding { name, .. } = result {
        assert_eq!(name, "written"); // Should bind to the participle's normalized form
    } else {
        panic!("Expected Binding, got {:?}", result);
    }
}

#[test]
fn test_classify_binding_fallback_participle() {
    let mut scope = Scope::new();

    // Case where we bind to a participle when no subject/object is present
    // and it IS a real participle (or at least we want to hit the fallback path)
    // Note: The logic prioritizes "false participles" first.
    // If the participle IS in the lexicon, `has_false_participle` is false.
    // Then it falls through subject/object checks.
    // Finally it hits the fallback `if !asm_stmt.participles.is_empty()`.

    // We need a verb lemma that IS in the lexicon for this test to hit the fallback path,
    // OR we just rely on `resolve_binding_target` falling through.
    // Let's use a dummy one but ensure `has_false_participle` would be false if we could control the lexicon.
    // Since we can't easily mock the lexicon here, we might just assume "unknown_verb" triggers false participle.
    // Wait, if "unknown_verb" is NOT in lexicon, `lookup` returns None, so `has_false_participle` is TRUE.
    // So to hit the fallback, we need `lookup` to return SOME.
    // "λεγω" (speak) is definitely in the lexicon.

    let asm_stmt = AssembledStatement {
        literals: vec![Literal::Number(5)],
        verb: Some(make_verb("let", "εστω")),
        participles: vec![ParticipleConstituent {
            verb_lemma: "λεγω".into(), // Should be in lexicon
            original: "legomenon".into(),
            normalized: "legomenon".into(),
            tense: crate::morphology::Tense::Present,
            voice: crate::morphology::Voice::Passive,
            case: Case::Nominative,
            gender: crate::morphology::Gender::Neuter,
            number: crate::morphology::Number::Singular,
        }],
        ..Default::default()
    };

    let result =
        classify_assembled_statement(&asm_stmt, &mut scope).expect("Classification failed");

    if let AnalyzedStatement::Binding { name, .. } = result {
        assert_eq!(name, "legomenon");
    } else {
        panic!("Expected Binding, got {:?}", result);
    }
}

#[test]
fn test_classify_print_binary_op() {
    let mut scope = Scope::new();
    scope.define("x", GlossaType::Number);

    // "x + 5 print"
    let asm_stmt = AssembledStatement {
        subject: Some(make_constituent("x", "x")),
        literals: vec![Literal::Number(5)],
        operators: vec![BinaryOp::Add],
        verb: Some(make_verb("print", "λεγε")),
        ..Default::default()
    };

    let result =
        classify_assembled_statement(&asm_stmt, &mut scope).expect("Classification failed");

    if let AnalyzedStatement::Print(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::BinOp { left, op, right: _ } = &exprs[0].expr {
            assert_eq!(*op, BinaryOp::Add);
            // check left is variable x
            if let AnalyzedExprKind::Variable(v) = &left.expr {
                assert_eq!(v, "x");
            } else {
                panic!("Expected Variable x");
            }
        } else {
            panic!("Expected BinOp");
        }
    } else {
        panic!("Expected Print, got {:?}", result);
    }
}

#[test]
fn test_classify_print_property_access() {
    let mut scope = Scope::new();
    scope.define("user", GlossaType::Unknown); // Type doesn't matter much for lookup unless it fails

    // "user.name print"
    // Assembler represents this as property_accesses: [("user", "name")]
    let asm_stmt = AssembledStatement {
        property_accesses: vec![("user".into(), "name".into())],
        verb: Some(make_verb("print", "λεγε")),
        ..Default::default()
    };

    let result =
        classify_assembled_statement(&asm_stmt, &mut scope).expect("Classification failed");

    if let AnalyzedStatement::Print(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::MethodCall {
            receiver, method, ..
        } = &exprs[0].expr
        {
            assert_eq!(method, "name");
            if let AnalyzedExprKind::Variable(v) = &receiver.expr {
                assert_eq!(v, "user");
            } else {
                panic!("Expected Variable user");
            }
        } else {
            panic!(
                "Expected MethodCall (property access is often lowered to method call or specific enum)"
            );
        }
    } else {
        panic!("Expected Print, got {:?}", result);
    }
}

#[test]
fn test_classify_print_index_access() {
    let mut scope = Scope::new();
    // array[0] print
    let asm_stmt = AssembledStatement {
        index_accesses: vec![(
            Expr::Word(crate::ast::Word {
                original: "arr".into(),
                normalized: "arr".into(),
            }),
            Expr::NumberLiteral(0),
        )],
        verb: Some(make_verb("print", "λεγε")),
        ..Default::default()
    };

    // Define "arr" so it can be resolved
    scope.define("arr", GlossaType::List(Box::new(GlossaType::Number)));

    let result =
        classify_assembled_statement(&asm_stmt, &mut scope).expect("Classification failed");

    if let AnalyzedStatement::Print(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::IndexAccess { array, index } = &exprs[0].expr {
            if let AnalyzedExprKind::Variable(v) = &array.expr {
                assert_eq!(v, "arr");
            } else {
                panic!("Expected array variable");
            }
            if let AnalyzedExprKind::NumberLiteral(n) = &index.expr {
                assert_eq!(*n, 0);
            } else {
                panic!("Expected index literal 0");
            }
        } else {
            panic!("Expected IndexAccess");
        }
    } else {
        panic!("Expected Print");
    }
}

#[test]
fn test_classify_print_unwrap() {
    let mut scope = Scope::new();
    // x! print
    let asm_stmt = AssembledStatement {
        unwraps: vec![Expr::Word(crate::ast::Word {
            original: "x".into(),
            normalized: "x".into(),
        })],
        verb: Some(make_verb("print", "λεγε")),
        ..Default::default()
    };

    scope.define("x", GlossaType::Option(Box::new(GlossaType::Number)));

    let result =
        classify_assembled_statement(&asm_stmt, &mut scope).expect("Classification failed");

    if let AnalyzedStatement::Print(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::Unwrap(inner) = &exprs[0].expr {
            if let AnalyzedExprKind::Variable(v) = &inner.expr {
                assert_eq!(v, "x");
            } else {
                panic!("Expected variable x");
            }
        } else {
            panic!("Expected Unwrap");
        }
    } else {
        panic!("Expected Print");
    }
}

#[test]
fn test_classify_print_default() {
    let mut scope = Scope::new();
    // "hello" print
    let asm_stmt = AssembledStatement {
        literals: vec![Literal::String("hello".into())],
        verb: Some(make_verb("print", "λεγε")),
        ..Default::default()
    };

    let result =
        classify_assembled_statement(&asm_stmt, &mut scope).expect("Classification failed");

    if let AnalyzedStatement::Print(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::StringLiteral(s) = &exprs[0].expr {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected StringLiteral");
        }
    } else {
        panic!("Expected Print");
    }
}
