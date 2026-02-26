use super::conversion::{convert_assembled_to_analyzed, extract_value};
use crate::ast::Expr;
use crate::morphology::lexicon::BinaryOp;
use crate::morphology::{Case, Number, Person, Tense};
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
        mood: None,
        voice: None,
    }
}

#[test]
fn test_classify_pop() {
    let mut scope = Scope::new();
    scope.define("stack", GlossaType::List(Box::new(GlossaType::Number)));

    // "stack pulls itself" (stack.pop())
    let asm_stmt = AssembledStatement {
        verb: Some(make_verb("ἕλκεται", "ελκω")),
        subject: Some(make_constituent("stack", "stack")),
        ..Default::default()
    };

    let analyzed =
        convert_assembled_to_analyzed(&asm_stmt, &mut scope).expect("Should classify pop");

    if let AnalyzedStatement::Expression(exprs) = analyzed {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } = &exprs[0].expr
        {
            assert_eq!(method, "pop");
            assert!(args.is_empty());
            if let AnalyzedExprKind::Variable(name) = &receiver.expr {
                assert_eq!(name, "stack");
            } else {
                panic!("Receiver should be variable");
            }
        } else {
            panic!("Expected MethodCall");
        }
    } else {
        panic!("Expected Expression statement");
    }
}

#[test]
fn test_classify_push_literal() {
    let mut scope = Scope::new();
    scope.define("stack", GlossaType::List(Box::new(GlossaType::Number)));

    // "stack pushes 42"
    let asm_stmt = AssembledStatement {
        verb: Some(make_verb("ὠθεῖ", "ωθεω")),
        subject: Some(make_constituent("stack", "stack")),
        literals: vec![Literal::Number(42)],
        ..Default::default()
    };

    let analyzed =
        convert_assembled_to_analyzed(&asm_stmt, &mut scope).expect("Should classify push");

    if let AnalyzedStatement::Expression(exprs) = analyzed {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
            assert_eq!(method, "push");
            assert_eq!(args.len(), 1);
            if let AnalyzedExprKind::NumberLiteral(n) = &args[0].expr {
                assert_eq!(*n, 42);
            } else {
                panic!("Arg should be literal 42");
            }
        } else {
            panic!("Expected MethodCall");
        }
    } else {
        panic!("Expected Expression statement");
    }
}

#[test]
fn test_classify_push_object() {
    let mut scope = Scope::new();
    scope.define("stack", GlossaType::List(Box::new(GlossaType::Number)));
    scope.define("val", GlossaType::Number);

    // "stack pushes val" (val is object)
    let asm_stmt = AssembledStatement {
        verb: Some(make_verb("ὠθεῖ", "ωθεω")),
        subject: Some(make_constituent("stack", "stack")),
        object: Some(make_constituent("val", "val")),
        ..Default::default()
    };

    let analyzed =
        convert_assembled_to_analyzed(&asm_stmt, &mut scope).expect("Should classify push object");

    if let AnalyzedStatement::Expression(exprs) = analyzed {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
            assert_eq!(method, "push");
            assert_eq!(args.len(), 1);
            if let AnalyzedExprKind::Variable(name) = &args[0].expr {
                assert_eq!(name, "val");
            } else {
                panic!("Arg should be variable val");
            }
        } else {
            panic!("Expected MethodCall");
        }
    } else {
        panic!("Expected Expression statement");
    }
}

#[test]
fn test_classify_push_default() {
    let mut scope = Scope::new();
    scope.define("stack", GlossaType::List(Box::new(GlossaType::Number)));

    // "stack pushes" (no object/literal -> default 0)
    let asm_stmt = AssembledStatement {
        verb: Some(make_verb("ὠθεῖ", "ωθεω")),
        subject: Some(make_constituent("stack", "stack")),
        ..Default::default()
    };

    let analyzed =
        convert_assembled_to_analyzed(&asm_stmt, &mut scope).expect("Should classify push default");

    if let AnalyzedStatement::Expression(exprs) = analyzed {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
            assert_eq!(method, "push");
            assert_eq!(args.len(), 1);
            if let AnalyzedExprKind::NumberLiteral(n) = &args[0].expr {
                assert_eq!(*n, 0);
            } else {
                panic!("Arg should be default 0");
            }
        } else {
            panic!("Expected MethodCall");
        }
    } else {
        panic!("Expected Expression statement");
    }
}

#[test]
fn test_classify_insert_map() {
    let mut scope = Scope::new();
    // Map<String, Number>
    scope.define(
        "map",
        GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number)),
    );

    // "map puts 'key' 100"
    let asm_stmt = AssembledStatement {
        verb: Some(make_verb("τίθησι", "τιθημι")),
        subject: Some(make_constituent("map", "map")),
        literals: vec![Literal::String("key".into()), Literal::Number(100)],
        ..Default::default()
    };

    let analyzed =
        convert_assembled_to_analyzed(&asm_stmt, &mut scope).expect("Should classify insert");

    if let AnalyzedStatement::Expression(exprs) = analyzed {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
            assert_eq!(method, "insert");
            assert_eq!(args.len(), 2);
            // Verify args...
        } else {
            panic!("Expected MethodCall");
        }
    } else {
        panic!("Expected Expression statement");
    }
}

#[test]
fn test_classify_insert_set() {
    let mut scope = Scope::new();
    // Set<String>
    scope.define("set", GlossaType::Set(Box::new(GlossaType::String)));

    // "set puts 'val'"
    let asm_stmt = AssembledStatement {
        verb: Some(make_verb("τίθησι", "τιθημι")),
        subject: Some(make_constituent("set", "set")),
        literals: vec![Literal::String("val".into())],
        ..Default::default()
    };

    let analyzed =
        convert_assembled_to_analyzed(&asm_stmt, &mut scope).expect("Should classify insert set");

    if let AnalyzedStatement::Expression(exprs) = analyzed {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
            assert_eq!(method, "insert");
            assert_eq!(args.len(), 1);
            if let AnalyzedExprKind::StringLiteral(s) = &args[0].expr {
                assert_eq!(s, "val");
            } else {
                panic!("Arg should be string literal");
            }
        } else {
            panic!("Expected MethodCall");
        }
    } else {
        panic!("Expected Expression statement");
    }
}

#[test]
fn test_extract_ok() {
    let scope = Scope::new();
    // Subject "ἐπιτυχία" (Ok), Literal 42
    let asm_stmt = AssembledStatement {
        subject: Some(make_constituent("ἐπιτυχία", "επιτυχια")),
        literals: vec![Literal::Number(42)],
        ..Default::default()
    };

    let (analyzed, glossa_type) = extract_value(&asm_stmt, &scope).expect("Should extract Ok");

    if let AnalyzedExprKind::Ok(inner) = analyzed.expr {
        if let AnalyzedExprKind::NumberLiteral(n) = inner.expr {
            assert_eq!(n, 42);
        } else {
            panic!("Expected number inside Ok");
        }
    } else {
        panic!("Expected Ok expression");
    }

    assert!(matches!(glossa_type, GlossaType::Result(_, _)));
}

#[test]
fn test_extract_err() {
    let scope = Scope::new();
    // Subject "σφάλμα" (Err), Literal 1 (error code)
    let asm_stmt = AssembledStatement {
        subject: Some(make_constituent("σφάλμα", "σφαλμα")),
        literals: vec![Literal::Number(1)],
        ..Default::default()
    };

    let (analyzed, glossa_type) = extract_value(&asm_stmt, &scope).expect("Should extract Err");

    if let AnalyzedExprKind::Err(inner) = analyzed.expr {
        if let AnalyzedExprKind::NumberLiteral(n) = inner.expr {
            assert_eq!(n, 1);
        } else {
            panic!("Expected number inside Err");
        }
    } else {
        panic!("Expected Err expression");
    }

    assert!(matches!(glossa_type, GlossaType::Result(_, _)));
}

#[test]
fn test_extract_unwrap() {
    let mut scope = Scope::new();

    // Simulate `x!` where x is a variable
    let expr = Expr::Word(crate::ast::Word {
        original: "x".into(),
        normalized: "x".into(),
    });

    let asm_stmt = AssembledStatement {
        unwraps: vec![expr],
        ..Default::default()
    };

    // Define x in scope so analyze_argument_expr works
    scope.define("x", GlossaType::Option(Box::new(GlossaType::Number)));

    let (analyzed, _) = extract_value(&asm_stmt, &scope).expect("Should extract unwrap");

    assert!(matches!(analyzed.expr, AnalyzedExprKind::Unwrap(_)));
}

#[test]
fn test_extract_subject_none() {
    let scope = Scope::new();
    // Subject "οὐδέν" (None)
    let asm_stmt = AssembledStatement {
        subject: Some(make_constituent("οὐδέν", "ουδεν")),
        ..Default::default()
    };

    let (analyzed, glossa_type) = extract_value(&asm_stmt, &scope).expect("Should extract None");

    assert!(matches!(analyzed.expr, AnalyzedExprKind::None));
    assert!(matches!(glossa_type, GlossaType::Option(_)));
}

#[test]
fn test_extract_subject_some() {
    let scope = Scope::new();
    // Subject "τί" (Some), Literal value for Some(val)
    let asm_stmt = AssembledStatement {
        subject: Some(make_constituent("τί", "τι")),
        literals: vec![Literal::Number(42)],
        ..Default::default()
    };

    let (analyzed, glossa_type) = extract_value(&asm_stmt, &scope).expect("Should extract Some");

    if let AnalyzedExprKind::Some(inner) = analyzed.expr {
        if let AnalyzedExprKind::NumberLiteral(n) = inner.expr {
            assert_eq!(n, 42);
        } else {
            panic!("Expected number inside Some");
        }
    } else {
        panic!("Expected Some expression");
    }

    assert!(matches!(glossa_type, GlossaType::Option(_)));
}

#[test]
fn test_extract_literal() {
    let scope = Scope::new();
    let asm_stmt = AssembledStatement {
        literals: vec![Literal::Number(123)],
        ..Default::default()
    };

    let (analyzed, glossa_type) = extract_value(&asm_stmt, &scope).expect("Should extract literal");

    if let AnalyzedExprKind::NumberLiteral(n) = analyzed.expr {
        assert_eq!(n, 123);
    } else {
        panic!("Expected NumberLiteral");
    }

    assert_eq!(glossa_type, GlossaType::Number);
}

#[test]
fn test_extract_binary_op_nominative_and_nominative() {
    let mut scope = Scope::new();
    scope.define("a", GlossaType::Number);
    scope.define("b", GlossaType::Number);

    // Simulate "a + b" where both are nominatives (no Object/Subject used)
    // Object: None
    // Nominatives: ["a", "b"]
    // Operator: "+"
    let asm_stmt = AssembledStatement {
        object: None,
        nominatives: vec![make_constituent("a", "a"), make_constituent("b", "b")],
        operators: vec![BinaryOp::Add],
        ..Default::default()
    };

    let (analyzed, glossa_type) =
        extract_value(&asm_stmt, &scope).expect("Should extract binary op for nom+nom");

    match analyzed.expr {
        AnalyzedExprKind::BinOp { left, op, right } => {
            assert_eq!(op, BinaryOp::Add);
            if let AnalyzedExprKind::Variable(name) = left.expr {
                assert_eq!(name, "a");
            } else {
                panic!("Left operand should be variable a");
            }
            if let AnalyzedExprKind::Variable(name) = right.expr {
                assert_eq!(name, "b");
            } else {
                panic!("Right operand should be variable b");
            }
        }
        _ => panic!("Expected BinOp, got {:?}", analyzed.expr),
    }

    assert_eq!(glossa_type, GlossaType::Number);
}

#[test]
fn test_extract_binary_op_object_and_literal() {
    let mut scope = Scope::new();
    scope.define("x", GlossaType::Number);

    // Simulate "x + 5"
    // Object: "x"
    // Literal: 5
    // Operator: "+"
    let asm_stmt = AssembledStatement {
        object: Some(make_constituent("x", "x")),
        literals: vec![Literal::Number(5)],
        operators: vec![BinaryOp::Add],
        ..Default::default()
    };

    let (analyzed, glossa_type) =
        extract_value(&asm_stmt, &scope).expect("Should extract binary op for obj+lit");

    match analyzed.expr {
        AnalyzedExprKind::BinOp { left, op, right } => {
            assert_eq!(op, BinaryOp::Add);
            if let AnalyzedExprKind::Variable(name) = left.expr {
                assert_eq!(name, "x");
            } else {
                panic!("Left operand should be variable x");
            }
            if let AnalyzedExprKind::NumberLiteral(val) = right.expr {
                assert_eq!(val, 5);
            } else {
                panic!("Right operand should be literal 5");
            }
        }
        _ => panic!("Expected BinOp, got {:?}", analyzed.expr),
    }

    assert_eq!(glossa_type, GlossaType::Number);
}

#[test]
fn test_extract_binary_op_object_and_nominative() {
    let mut scope = Scope::new();
    scope.define("x", GlossaType::Number);
    scope.define("y", GlossaType::Number);

    // Simulate "x + y"
    // Object: "x"
    // Nominative: "y" (extra nominative, not subject)
    // Operator: "+"
    let asm_stmt = AssembledStatement {
        object: Some(make_constituent("x", "x")),
        nominatives: vec![make_constituent("y", "y")],
        operators: vec![BinaryOp::Add],
        ..Default::default()
    };

    let (analyzed, glossa_type) =
        extract_value(&asm_stmt, &scope).expect("Should extract binary op");

    match analyzed.expr {
        AnalyzedExprKind::BinOp { left, op, right } => {
            assert_eq!(op, BinaryOp::Add);
            if let AnalyzedExprKind::Variable(name) = left.expr {
                assert_eq!(name, "x");
            } else {
                panic!("Left operand should be variable x");
            }
            if let AnalyzedExprKind::Variable(name) = right.expr {
                assert_eq!(name, "y");
            } else {
                panic!("Right operand should be variable y");
            }
        }
        AnalyzedExprKind::Variable(name) => {
            panic!(
                "Expected BinOp, got Variable '{}' (Operator Drop Bug!)",
                name
            );
        }
        _ => panic!("Expected BinOp, got {:?}", analyzed.expr),
    }

    assert_eq!(glossa_type, GlossaType::Number);
}

#[test]
fn test_extract_object_variable() {
    let mut scope = Scope::new();
    scope.define("foo", GlossaType::String);

    let asm_stmt = AssembledStatement {
        object: Some(make_constituent("foo", "foo")),
        ..Default::default()
    };

    let (analyzed, _) =
        extract_value(&asm_stmt, &scope).expect("Should extract variable from object");

    if let AnalyzedExprKind::Variable(name) = analyzed.expr {
        assert_eq!(name, "foo");
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_extract_object_none() {
    let scope = Scope::new();
    // Object "οὐδέν" (None) - checking if logic handles object as enum variant
    let asm_stmt = AssembledStatement {
        object: Some(make_constituent("οὐδέν", "ουδεν")),
        ..Default::default()
    };

    let (analyzed, _) = extract_value(&asm_stmt, &scope).expect("Should extract None from object");

    assert!(matches!(analyzed.expr, AnalyzedExprKind::None));
}

#[test]
fn test_extract_object_some() {
    let scope = Scope::new();
    // Object "τί" (Some)
    let asm_stmt = AssembledStatement {
        object: Some(make_constituent("τί", "τι")),
        literals: vec![Literal::Number(10)],
        ..Default::default()
    };

    let (analyzed, _) = extract_value(&asm_stmt, &scope).expect("Should extract Some from object");

    if let AnalyzedExprKind::Some(inner) = analyzed.expr {
        if let AnalyzedExprKind::NumberLiteral(n) = inner.expr {
            assert_eq!(n, 10);
        } else {
            panic!("Expected NumberLiteral");
        }
    } else {
        panic!("Expected Some expression");
    }
}

#[test]
fn test_extract_object_numeral() {
    let scope = Scope::new();
    // Object "πέντε" (five)
    let asm_stmt = AssembledStatement {
        object: Some(make_constituent("πέντε", "πεντε")),
        ..Default::default()
    };

    let (analyzed, glossa_type) =
        extract_value(&asm_stmt, &scope).expect("Should extract numeral from object");

    if let AnalyzedExprKind::NumberLiteral(n) = analyzed.expr {
        assert_eq!(n, 5);
    } else {
        panic!("Expected NumberLiteral for numeral word");
    }

    assert_eq!(glossa_type, GlossaType::Number);
}

#[test]
fn test_default_case() {
    let scope = Scope::new();
    let asm_stmt = AssembledStatement::default();

    let (analyzed, glossa_type) =
        extract_value(&asm_stmt, &scope).expect("Should fallback to default");

    if let AnalyzedExprKind::NumberLiteral(n) = analyzed.expr {
        assert_eq!(n, 0);
    } else {
        panic!("Expected default 0");
    }

    assert_eq!(glossa_type, GlossaType::Number);
}
