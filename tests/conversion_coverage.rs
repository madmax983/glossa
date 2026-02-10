use glossa::ast::{Expr, Word};
use glossa::morphology::{Case, Gender};
use glossa::semantic::assembled::{AssembledStatement, Constituent, Literal, VerbConstituent};
use glossa::semantic::conversion::{convert_assembled_to_analyzed, extract_value};
use glossa::semantic::{AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope};

fn constituent(lemma: &str) -> Constituent {
    Constituent {
        lemma: lemma.into(),
        original: lemma.into(),
        case: Case::Nominative,
        number: None,
        gender: None,
        person: None,
    }
}

fn verb(lemma: &str) -> VerbConstituent {
    VerbConstituent {
        lemma: lemma.into(),
        original: lemma.into(),
        person: None,
        number: None,
        voice: None,
        mood: None,
        tense: None,
    }
}

fn number_literal(val: i64) -> Literal {
    Literal::Number(val)
}

fn word_expr(text: &str) -> Expr {
    Expr::Word(Word::new(text))
}

#[test]
fn test_statement_classification_genitive_method_call() {
    // Test: obj.method (implicitly, without print)
    // Structure: Subject="method", Genitives=["obj"], Verb=None (or non-print)
    let mut scope = Scope::new();
    scope.define_type(
        "MyType",
        GlossaType::Struct {
            name: "MyType".into(),
            fields: vec![],
            gender: Gender::Neuter,
        },
    );
    scope.define(
        "obj".to_string(),
        GlossaType::Struct {
            name: "MyType".into(),
            fields: vec![],
            gender: Gender::Neuter,
        },
    );

    let asm = AssembledStatement {
        subject: Some(constituent("method")),
        genitives: vec![constituent("obj")],
        ..Default::default()
    };

    let result = convert_assembled_to_analyzed(&asm, &mut scope).expect("Should convert");

    if let AnalyzedStatement::Expression(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } = &exprs[0].expr
        {
            assert_eq!(method, "method");
            assert!(args.is_empty());
            if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                assert_eq!(name, "obj");
            } else {
                panic!("Expected variable receiver");
            }
        } else {
            panic!("Expected MethodCall expression");
        }
    } else {
        panic!("Expected Expression statement");
    }
}

#[test]
fn test_statement_classification_equality_assertion() {
    // Test: val1 5 ἰσοῦται
    let mut scope = Scope::new();
    scope.define("val1".to_string(), GlossaType::Number);

    let asm = AssembledStatement {
        verb: Some(verb("ἰσοῦται")),
        subject: Some(constituent("val1")),
        literals: vec![number_literal(5)],
        ..Default::default()
    };

    let result = convert_assembled_to_analyzed(&asm, &mut scope).expect("Should convert");

    if let AnalyzedStatement::Expression(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::AssertEq { left, right } = &exprs[0].expr {
            if let AnalyzedExprKind::Variable(name) = &left.expr {
                assert_eq!(name, "val1");
            } else {
                panic!("Expected variable left");
            }
            if let AnalyzedExprKind::NumberLiteral(val) = &right.expr {
                assert_eq!(*val, 5);
            } else {
                panic!("Expected literal right");
            }
        } else {
            panic!("Expected AssertEq expression");
        }
    } else {
        panic!("Expected Expression statement");
    }
}

#[test]
fn test_statement_classification_assertion() {
    // Test: val in col δεῖ
    // Structure: Verb="δεῖ", Subject="col", Literals=[val], Preposition=True
    let mut scope = Scope::new();
    scope.define(
        "col".to_string(),
        GlossaType::List(Box::new(GlossaType::Number)),
    );

    let asm = AssembledStatement {
        verb: Some(verb("δεῖ")),
        subject: Some(constituent("col")),
        literals: vec![number_literal(42)],
        has_containment_preposition: true,
        ..Default::default()
    };

    let result = convert_assembled_to_analyzed(&asm, &mut scope).expect("Should convert");

    if let AnalyzedStatement::Expression(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::Assert { condition } = &exprs[0].expr {
            if let AnalyzedExprKind::MethodCall {
                receiver,
                method,
                args,
            } = &condition.expr
            {
                assert_eq!(method, "contains");
                assert_eq!(args.len(), 1);
                if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                    assert_eq!(name, "col");
                }
            } else {
                panic!("Expected contains call");
            }
        } else {
            panic!("Expected Assert expression");
        }
    } else {
        panic!("Expected Expression statement");
    }
}

#[test]
fn test_statement_classification_property_access_print() {
    // Test: user.name print (λέγε)
    // Structure: Verb="λέγε", Subject="name", Genitives=["user"]
    let mut scope = Scope::new();
    scope.define_type(
        "User",
        GlossaType::Struct {
            name: "User".into(),
            fields: vec![],
            gender: Gender::Masculine,
        },
    );
    scope.define(
        "user".to_string(),
        GlossaType::Struct {
            name: "User".into(),
            fields: vec![],
            gender: Gender::Masculine,
        },
    );

    let asm = AssembledStatement {
        verb: Some(verb("λέγε")),
        subject: Some(constituent("name")),
        genitives: vec![constituent("user")],
        ..Default::default()
    };

    let result = convert_assembled_to_analyzed(&asm, &mut scope).expect("Should convert");

    if let AnalyzedStatement::Print(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::PropertyAccess { owner, property } = &exprs[0].expr {
            assert_eq!(property, "name");
            if let AnalyzedExprKind::Variable(name) = &owner.expr {
                assert_eq!(name, "user");
            } else {
                panic!("Expected variable owner");
            }
        } else {
            panic!("Expected PropertyAccess expression");
        }
    } else {
        panic!("Expected Print statement");
    }
}

#[test]
fn test_statement_classification_collection_push() {
    // Test: list 42 push (ὠθεῖ)
    let mut scope = Scope::new();
    scope.define(
        "list".to_string(),
        GlossaType::List(Box::new(GlossaType::Number)),
    );

    let asm = AssembledStatement {
        verb: Some(verb("ὠθεῖ")),
        subject: Some(constituent("list")),
        literals: vec![number_literal(42)],
        ..Default::default()
    };

    let result = convert_assembled_to_analyzed(&asm, &mut scope).expect("Should convert");

    if let AnalyzedStatement::Expression(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } = &exprs[0].expr
        {
            assert_eq!(method, "push");
            assert_eq!(args.len(), 1);
            if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                assert_eq!(name, "list");
            }
        } else {
            panic!("Expected MethodCall expression");
        }
    } else {
        panic!("Expected Expression statement");
    }
}

#[test]
fn test_statement_classification_function_call() {
    // Test: my_func 10 call (implies binding verb context or just call)
    // Actually classify_function_call looks for binding verbs + function name in Nom/Obj/Gen
    // Example: "x my_func 10 is" (binding)

    let mut scope = Scope::new();
    scope.define_function("my_func", vec![GlossaType::Number], Some(GlossaType::Number));

    // Verb must be binding verb (e.g. "ἔστω" or "εἶναι" or similar that maps to IS)
    // Let's use "ἔστω" (Let be)
    // Structure: Verb="ἔστω", Subject="res", Object="my_func", Literals=[10]
    let asm = AssembledStatement {
        verb: Some(verb("ἔστω")),
        subject: Some(constituent("res")),
        object: Some(constituent("my_func")),
        literals: vec![number_literal(10)],
        ..Default::default()
    };

    let result = convert_assembled_to_analyzed(&asm, &mut scope).expect("Should convert");

    if let AnalyzedStatement::Binding { name, value, .. } = result {
        assert_eq!(name, "res");
        if let AnalyzedExprKind::FunctionCall { func, args } = value.expr {
            assert_eq!(func, "my_func");
            assert_eq!(args.len(), 1);
        } else {
            panic!("Expected FunctionCall value");
        }
    } else {
        panic!("Expected Binding statement");
    }
}

#[test]
fn test_extract_value_unwrap_priority() {
    let scope = Scope::new();

    // Add unwrap expression (Highest priority)
    let asm = AssembledStatement {
        unwraps: vec![Expr::NumberLiteral(42)],
        // Add conflicting lower priority items
        literals: vec![number_literal(100)],    // Should be ignored
        object: Some(constituent("ignore_me")), // Should be ignored
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    if let AnalyzedExprKind::Unwrap(inner) = expr.expr {
        if let AnalyzedExprKind::NumberLiteral(val) = inner.expr {
            assert_eq!(val, 42);
        } else {
            panic!("Expected inner NumberLiteral");
        }
    } else {
        panic!("Expected Unwrap expression, got {:?}", expr.expr);
    }
    assert_eq!(ty, GlossaType::Unknown);
}

#[test]
fn test_extract_value_enum_subject_none() {
    let scope = Scope::new();
    let asm = AssembledStatement {
        subject: Some(constituent("οὐδέν")),
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    assert!(matches!(expr.expr, AnalyzedExprKind::None));
    assert!(matches!(ty, GlossaType::Option(_)));
}

#[test]
fn test_extract_value_enum_subject_some() {
    let scope = Scope::new();
    let asm = AssembledStatement {
        subject: Some(constituent("τί")),
        literals: vec![number_literal(42)],
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    if let AnalyzedExprKind::Some(inner) = expr.expr {
        if let AnalyzedExprKind::NumberLiteral(val) = inner.expr {
            assert_eq!(val, 42);
        } else {
            panic!("Expected inner NumberLiteral");
        }
    } else {
        panic!("Expected Some expression");
    }
    assert!(matches!(ty, GlossaType::Option(_)));
}

#[test]
fn test_extract_value_property_access() {
    let scope = Scope::new();

    let asm = AssembledStatement {
        property_accesses: vec![("user".to_string(), "name".to_string())],
        // Add conflicting lower priority items
        literals: vec![number_literal(100)], // Should be ignored
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    if let AnalyzedExprKind::MethodCall {
        receiver,
        method,
        args,
    } = expr.expr
    {
        assert_eq!(method, "name");
        assert!(args.is_empty());
        if let AnalyzedExprKind::Variable(name) = receiver.expr {
            assert_eq!(name, "user");
        } else {
            panic!("Expected variable receiver");
        }
    } else {
        panic!("Expected MethodCall expression (property access is lowered to method call)");
    }
    // Property access currently returns Number type as placeholder
    assert_eq!(ty, GlossaType::Number);
}

#[test]
fn test_extract_value_index_access() {
    let mut scope = Scope::new();
    // Define 'arr' so it can be resolved
    scope.define(
        "arr".to_string(),
        GlossaType::List(Box::new(GlossaType::Number)),
    );

    let asm = AssembledStatement {
        index_accesses: vec![(word_expr("arr"), Expr::NumberLiteral(0))],
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    if let AnalyzedExprKind::IndexAccess { array, index } = expr.expr {
        if let AnalyzedExprKind::Variable(name) = array.expr {
            assert_eq!(name, "arr");
        } else {
            panic!("Expected variable array");
        }
        if let AnalyzedExprKind::NumberLiteral(val) = index.expr {
            assert_eq!(val, 0);
        } else {
            panic!("Expected literal index");
        }
    } else {
        panic!("Expected IndexAccess expression");
    }
    assert_eq!(ty, GlossaType::Unknown);
}

#[test]
fn test_extract_value_array_literal() {
    let scope = Scope::new();

    let asm = AssembledStatement {
        arrays: vec![vec![Expr::NumberLiteral(1), Expr::NumberLiteral(2)]],
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    if let AnalyzedExprKind::ArrayLiteral(elements) = expr.expr {
        assert_eq!(elements.len(), 2);
    } else {
        panic!("Expected ArrayLiteral expression");
    }
    assert!(matches!(ty, GlossaType::List(_)));
}

#[test]
fn test_extract_value_binary_op() {
    let scope = Scope::new();

    let asm = AssembledStatement {
        literals: vec![number_literal(1), number_literal(2)],
        operators: vec![glossa::morphology::lexicon::BinaryOp::Add],
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    if let AnalyzedExprKind::BinOp {
        left: _,
        op,
        right: _,
    } = expr.expr
    {
        assert_eq!(op, glossa::morphology::lexicon::BinaryOp::Add);
    } else {
        panic!("Expected BinOp expression");
    }
    // Binary ops on numbers produce number
    assert_eq!(ty, GlossaType::Number);
}

#[test]
fn test_extract_value_literal() {
    let scope = Scope::new();

    let asm = AssembledStatement {
        literals: vec![number_literal(42)],
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    if let AnalyzedExprKind::NumberLiteral(val) = expr.expr {
        assert_eq!(val, 42);
    } else {
        panic!("Expected NumberLiteral expression");
    }
    assert_eq!(ty, GlossaType::Number);
}

#[test]
fn test_extract_value_object_variable() {
    let scope = Scope::new();

    let asm = AssembledStatement {
        object: Some(constituent("x")),
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    if let AnalyzedExprKind::Variable(name) = expr.expr {
        assert_eq!(name, "x");
    } else {
        panic!("Expected Variable expression");
    }
    assert_eq!(ty, GlossaType::Unknown);
}

#[test]
fn test_extract_value_object_none() {
    let scope = Scope::new();

    let asm = AssembledStatement {
        object: Some(constituent("οὐδέν")),
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    assert!(matches!(expr.expr, AnalyzedExprKind::None));
    assert!(matches!(ty, GlossaType::Option(_)));
}

#[test]
fn test_extract_value_fallback_default() {
    let scope = Scope::new();
    let asm = AssembledStatement::default(); // Empty statement

    let (expr, ty) = extract_value(&asm, &scope).expect("Should return default");

    // Should return 0 (Number) by default
    if let AnalyzedExprKind::NumberLiteral(val) = expr.expr {
        assert_eq!(val, 0);
    } else {
        panic!("Expected default NumberLiteral(0)");
    }
    assert_eq!(ty, GlossaType::Number);
}

#[test]
fn test_extract_value_detect_enum_ok() {
    let scope = Scope::new();
    let asm = AssembledStatement {
        subject: Some(constituent("ἐπιτυχία")), // Ok
        literals: vec![number_literal(42)],
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    if let AnalyzedExprKind::Ok(inner) = expr.expr {
        if let AnalyzedExprKind::NumberLiteral(val) = inner.expr {
            assert_eq!(val, 42);
        } else {
            panic!("Expected inner NumberLiteral");
        }
    } else {
        panic!("Expected Ok expression");
    }
    assert!(matches!(ty, GlossaType::Result(..)));
}

#[test]
fn test_extract_value_detect_enum_err() {
    let scope = Scope::new();
    let asm = AssembledStatement {
        subject: Some(constituent("σφάλμα")), // Err
        literals: vec![number_literal(1)],
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    if let AnalyzedExprKind::Err(inner) = expr.expr {
        if let AnalyzedExprKind::NumberLiteral(val) = inner.expr {
            assert_eq!(val, 1);
        } else {
            panic!("Expected inner NumberLiteral");
        }
    } else {
        panic!("Expected Err expression");
    }
    assert!(matches!(ty, GlossaType::Result(..)));
}

#[test]
fn test_extract_value_genitive_method_call() {
    let mut scope = Scope::new();
    scope.define_type(
        "MyType",
        GlossaType::Struct {
            name: "MyType".into(),
            fields: vec![],
            gender: Gender::Neuter,
        },
    );
    scope.define(
        "obj".to_string(),
        GlossaType::Struct {
            name: "MyType".into(),
            fields: vec![],
            gender: Gender::Neuter,
        },
    );

    let asm = AssembledStatement {
        subject: Some(constituent("method")),
        genitives: vec![constituent("obj")],
        literals: vec![number_literal(10)],
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    if let AnalyzedExprKind::MethodCall {
        receiver,
        method,
        args,
    } = expr.expr
    {
        assert_eq!(method, "method");
        assert_eq!(args.len(), 1);
        if let AnalyzedExprKind::Variable(name) = receiver.expr {
            assert_eq!(name, "obj");
        } else {
            panic!("Expected variable receiver");
        }
    } else {
        panic!("Expected MethodCall expression");
    }
    assert_eq!(ty, GlossaType::Unknown);
}

#[test]
fn test_extract_value_nominative_enum() {
    let scope = Scope::new();
    let asm = AssembledStatement {
        nominatives: vec![constituent("τί")], // Some
        literals: vec![number_literal(99)],
        ..Default::default()
    };

    let (expr, ty) = extract_value(&asm, &scope).expect("Should extract value");

    if let AnalyzedExprKind::Some(inner) = expr.expr {
        if let AnalyzedExprKind::NumberLiteral(val) = inner.expr {
            assert_eq!(val, 99);
        } else {
            panic!("Expected inner NumberLiteral");
        }
    } else {
        panic!("Expected Some expression from nominative");
    }
    assert!(matches!(ty, GlossaType::Option(_)));
}
