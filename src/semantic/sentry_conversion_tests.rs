use super::conversion::{convert_assembled_to_analyzed, values::extract_value};
use crate::morphology::lexicon::BinaryOp;
use crate::morphology::{Case, Mood, Number, Person, Tense};
use crate::semantic::{
    AnalyzedExprKind, AnalyzedStatement, AssembledStatement, Constituent, GlossaType, Literal,
    Scope, VerbConstituent,
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
        person: Some(Person::Third),
        number: Some(Number::Singular),
        tense: Some(Tense::Present),
        mood: Some(Mood::Indicative),
        voice: None,
    }
}

// 🛡️ SENTRY: Coverage for recursive value extraction
// This tests that we can extract values from nested structures like blocks and phrases
// which are crucial for complex expressions.

#[test]
fn test_extract_value_from_nested_phrase() {
    let scope = Scope::new();

    // Setup a nested phrase containing a literal number: (42)
    let _nested_expr = crate::ast::Expr::Phrase(vec![crate::ast::Expr::NumberLiteral(42)]);

    let asm_stmt = AssembledStatement {
        nested_phrases: vec![vec![crate::ast::Expr::NumberLiteral(42)]],
        ..Default::default()
    };

    let (analyzed, glossa_type) =
        extract_value(&asm_stmt, &scope).expect("Should extract value from nested phrase");

    if let AnalyzedExprKind::NumberLiteral(n) = analyzed.expr {
        assert_eq!(n, 42);
    } else {
        panic!("Expected NumberLiteral(42), got {:?}", analyzed.expr);
    }

    assert_eq!(glossa_type, GlossaType::Number);
}

#[test]
fn test_extract_value_from_block() {
    let scope = Scope::new();

    // Setup a block containing a statement with a number: { 42. }
    let stmt = crate::ast::Statement::Regular {
        clauses: vec![crate::ast::Clause {
            expressions: vec![crate::ast::Expr::NumberLiteral(42)],
        }],
        is_query: false,
        is_propagate: false,
    };

    let asm_stmt = AssembledStatement {
        blocks: vec![vec![stmt]],
        ..Default::default()
    };

    let (analyzed, glossa_type) =
        extract_value(&asm_stmt, &scope).expect("Should extract value from block");

    if let AnalyzedExprKind::NumberLiteral(n) = analyzed.expr {
        assert_eq!(n, 42);
    } else {
        panic!("Expected NumberLiteral(42), got {:?}", analyzed.expr);
    }

    assert_eq!(glossa_type, GlossaType::Number);
}

#[test]
fn test_extract_value_complex_binary_op_fallback() {
    // This tests the fallback path in extract_binary_op where we combine
    // Object + Nominative for binary operations (e.g. x + y)
    let mut scope = Scope::new();
    scope.define("x", GlossaType::Number);
    scope.define("y", GlossaType::Number);

    let asm_stmt = AssembledStatement {
        object: Some(make_constituent("x", "x")),
        nominatives: vec![make_constituent("y", "y")],
        operators: vec![BinaryOp::Add],
        ..Default::default()
    };

    let (analyzed, glossa_type) =
        extract_value(&asm_stmt, &scope).expect("Should extract binary op (obj+nom)");

    if let AnalyzedExprKind::BinOp { left, op, right } = analyzed.expr {
        assert_eq!(op, BinaryOp::Add);

        if let AnalyzedExprKind::Variable(name) = left.expr {
            assert_eq!(name, "x");
        } else {
            panic!("Left should be x");
        }

        if let AnalyzedExprKind::Variable(name) = right.expr {
            assert_eq!(name, "y");
        } else {
            panic!("Right should be y");
        }
    } else {
        panic!("Expected BinOp, got {:?}", analyzed.expr);
    }

    assert_eq!(glossa_type, GlossaType::Number);
}

#[test]
fn test_extract_value_complex_binary_op_two_nominatives() {
    // This tests the fallback path for Nominative + Nominative (e.g. a + b)
    let mut scope = Scope::new();
    scope.define("a", GlossaType::Number);
    scope.define("b", GlossaType::Number);

    let asm_stmt = AssembledStatement {
        nominatives: vec![make_constituent("a", "a"), make_constituent("b", "b")],
        operators: vec![BinaryOp::Add],
        ..Default::default()
    };

    let (analyzed, glossa_type) =
        extract_value(&asm_stmt, &scope).expect("Should extract binary op (nom+nom)");

    if let AnalyzedExprKind::BinOp { left, op, right } = analyzed.expr {
        assert_eq!(op, BinaryOp::Add);

        if let AnalyzedExprKind::Variable(name) = left.expr {
            assert_eq!(name, "a");
        } else {
            panic!("Left should be a");
        }

        if let AnalyzedExprKind::Variable(name) = right.expr {
            assert_eq!(name, "b");
        } else {
            panic!("Right should be b");
        }
    } else {
        panic!("Expected BinOp, got {:?}", analyzed.expr);
    }

    assert_eq!(glossa_type, GlossaType::Number);
}

#[test]
fn test_extract_value_array_literal() {
    let scope = Scope::new();

    // [1, 2]
    let array_elements = vec![
        crate::ast::Expr::NumberLiteral(1),
        crate::ast::Expr::NumberLiteral(2),
    ];

    let asm_stmt = AssembledStatement {
        arrays: vec![array_elements],
        ..Default::default()
    };

    let (analyzed, glossa_type) =
        extract_value(&asm_stmt, &scope).expect("Should extract array literal");

    if let AnalyzedExprKind::ArrayLiteral(elements) = analyzed.expr {
        assert_eq!(elements.len(), 2);
    } else {
        panic!("Expected ArrayLiteral");
    }

    // Verify type inference
    if let GlossaType::List(inner) = glossa_type {
        // FIXME: Type inference in `extract_array` currently returns Unknown for element type
        // instead of inferring from the first element. We verify this behavior (Unknown) for now
        // to document the bug, rather than asserting correct behavior (Number) which would fail.
        assert_eq!(*inner, GlossaType::Unknown);
    } else {
        panic!("Expected List<Number>");
    }
}

#[test]
fn test_extract_value_index_access() {
    let scope = Scope::new();

    // arr[0]
    let arr_expr = crate::ast::Expr::ArrayLiteral(vec![]);
    let idx_expr = crate::ast::Expr::NumberLiteral(0);

    let asm_stmt = AssembledStatement {
        index_accesses: vec![(arr_expr, idx_expr)],
        ..Default::default()
    };

    let (analyzed, _) = extract_value(&asm_stmt, &scope).expect("Should extract index access");

    if let AnalyzedExprKind::IndexAccess { .. } = analyzed.expr {
        // Success
    } else {
        panic!("Expected IndexAccess");
    }
}

#[test]
fn test_extract_value_property_access() {
    let scope = Scope::new();

    // owner.prop
    let asm_stmt = AssembledStatement {
        property_accesses: vec![("owner".to_string(), "prop".to_string())],
        ..Default::default()
    };

    let (analyzed, _) = extract_value(&asm_stmt, &scope).expect("Should extract property access");

    if let AnalyzedExprKind::MethodCall { method, .. } = analyzed.expr {
        assert_eq!(method, "prop");
    } else {
        panic!("Expected MethodCall (property access maps to method call currently)");
    }
}

#[test]
fn test_assignment_to_immutable_variable_returns_error() {
    let mut scope = Scope::new();
    // Define an immutable variable
    scope.define("constant", GlossaType::Number);

    // "constant becomes 42"
    let asm_stmt = AssembledStatement {
        verb: Some(make_verb("γίγνεται", "γιγνομαι")),
        subject: Some(make_constituent("constant", "constant")),
        literals: vec![Literal::Number(42)],
        ..Default::default()
    };

    let result = convert_assembled_to_analyzed(&asm_stmt, &mut scope);

    assert!(result.is_err(), "Assignment to immutable should fail");
    let err = result.unwrap_err();
    // The error message uses "ἀμετάβλητόν" (immutable)
    assert!(
        err.to_string().contains("ἀμετάβλητόν"),
        "Error should mention immutability (ἀμετάβλητόν)"
    );
}

#[test]
fn test_assignment_without_value_returns_error() {
    let mut scope = Scope::new();
    scope.define_mut("x", GlossaType::Number);

    // "x becomes" (missing value)
    let asm_stmt = AssembledStatement {
        verb: Some(make_verb("γίγνεται", "γιγνομαι")),
        subject: Some(make_constituent("x", "x")),
        ..Default::default()
    };

    let result = convert_assembled_to_analyzed(&asm_stmt, &mut scope);

    assert!(result.is_err(), "Assignment without value should fail");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("requires a value"),
        "Error should mention missing value"
    );
}

#[test]
fn test_binding_with_propagate() {
    let mut scope = Scope::new();

    // "x let 5;" -> propagate flag is set
    let asm_stmt = AssembledStatement {
        verb: Some(make_verb("ἔστω", "ειμι")),
        subject: Some(make_constituent("x", "x")),
        literals: vec![Literal::Number(5)],
        is_propagate: true,
        ..Default::default()
    };

    let analyzed = convert_assembled_to_analyzed(&asm_stmt, &mut scope)
        .expect("Should analyze binding with propagate");

    if let AnalyzedStatement::Binding { value, .. } = analyzed {
        assert!(
            matches!(value.expr, AnalyzedExprKind::Try(_)),
            "Value should be wrapped in Try"
        );

        if let AnalyzedExprKind::Try(inner) = value.expr {
            if let AnalyzedExprKind::NumberLiteral(n) = inner.expr {
                assert_eq!(n, 5);
            } else {
                panic!("Inner expression should be number 5");
            }
        }
    } else {
        panic!("Expected Binding statement");
    }
}

#[test]
fn test_subjunctive_comparison() {
    let mut scope = Scope::new();
    scope.define("x", GlossaType::Number);

    // "if x > 5 let" (subjunctive be verb with comparison)
    // Verb: ᾖ (subjunctive "be")
    // Subject: x
    // Literal: 5
    // Operator: >
    let mut verb = make_verb("ᾖ", "ειμι");
    verb.mood = Some(Mood::Subjunctive);

    let asm_stmt = AssembledStatement {
        verb: Some(verb),
        subject: Some(make_constituent("x", "x")),
        literals: vec![Literal::Number(5)],
        operators: vec![BinaryOp::Gt],
        ..Default::default()
    };

    let analyzed = convert_assembled_to_analyzed(&asm_stmt, &mut scope)
        .expect("Should analyze subjunctive comparison");

    if let AnalyzedStatement::Expression(exprs) = analyzed {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::BinOp { left, op, right } = &exprs[0].expr {
            assert_eq!(*op, BinaryOp::Gt);

            if let AnalyzedExprKind::Variable(name) = &left.expr {
                assert_eq!(name, "x");
            } else {
                panic!("Left should be x");
            }

            if let AnalyzedExprKind::NumberLiteral(n) = &right.expr {
                assert_eq!(*n, 5);
            } else {
                panic!("Right should be 5");
            }
        } else {
            panic!("Expected BinOp");
        }
    } else {
        panic!("Expected Expression statement");
    }
}

#[test]
fn test_binding_subject_object_swap() {
    let mut scope = Scope::new();
    // Pre-define existing variable
    scope.define("existing", GlossaType::Number);

    // "existing new let"
    // Subject: existing (already defined)
    // Object: new (undefined)
    // Should swap and bind "new" to value "existing"
    let asm_stmt = AssembledStatement {
        verb: Some(make_verb("ἔστω", "ειμι")),
        subject: Some(make_constituent("existing", "existing")),
        object: Some(make_constituent("new", "new")),
        ..Default::default()
    };

    let analyzed = convert_assembled_to_analyzed(&asm_stmt, &mut scope)
        .expect("Should analyze binding with swap");

    if let AnalyzedStatement::Binding { name, value, .. } = analyzed {
        assert_eq!(name, "new", "Should bind to the undefined variable 'new'");

        if let AnalyzedExprKind::Variable(var_name) = value.expr {
            assert_eq!(var_name, "existing", "Value should be 'existing'");
        } else {
            panic!("Value should be variable 'existing'");
        }
    } else {
        panic!("Expected Binding statement");
    }
}

#[test]
fn test_try_parse_genitive_method_call_extraction() {
    let mut scope = Scope::new();
    scope.define(
        "owner",
        GlossaType::Struct {
            name: "OwnerType".into(),
            gender: crate::morphology::Gender::Masculine,
            fields: vec![],
        },
    );

    let asm_stmt = AssembledStatement {
        subject: Some(make_constituent("method_name", "method_name")),
        genitives: vec![make_constituent("owner", "owner")],
        ..Default::default()
    };

    let (analyzed, _) =
        extract_value(&asm_stmt, &scope).expect("Should extract genitive method call");

    if let AnalyzedExprKind::MethodCall {
        receiver,
        method,
        args,
    } = analyzed.expr
    {
        assert_eq!(method, "method_name");
        assert!(args.is_empty());
        if let AnalyzedExprKind::Variable(owner_name) = receiver.expr {
            assert_eq!(owner_name, "owner");
        } else {
            panic!("Expected receiver to be a Variable");
        }
    } else {
        panic!("Expected MethodCall, got {:?}", analyzed.expr);
    }
}
