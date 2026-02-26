use super::conversion::extract_value;
use crate::ast::Expr;
use crate::morphology::lexicon::{BinaryOp, UnaryOp};
use crate::morphology::{Case, Number};
use crate::semantic::conversion::extract_binary_op;
use crate::semantic::{
    AnalyzedExprKind, AssembledStatement, Constituent, GlossaType, Literal, Scope,
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

#[test]
fn test_extract_binary_op_nominative_and_literal() {
    let mut scope = Scope::new();
    scope.define("nom", GlossaType::Number);

    // nominative (nom) + literal (1) -> op (+)
    let mut asm = AssembledStatement::default();
    asm.nominatives.push(Constituent {
        lemma: "nom".into(),
        original: "nom".into(),
        normalized: "nom".into(),
        case: Case::Nominative,
        number: Some(Number::Singular),
        gender: None,
        person: None,
    });
    asm.literals.push(Literal::Number(1));
    asm.operators.push(BinaryOp::Add);

    let result = extract_binary_op(&asm, &scope);
    assert!(result.is_ok());
    let extracted = result.unwrap();
    assert!(extracted.is_some());
    let (expr, ty) = extracted.unwrap();
    assert_eq!(ty, GlossaType::Number);

    if let AnalyzedExprKind::BinOp { left, op, right } = expr.expr {
        assert_eq!(op, BinaryOp::Add);
        if let AnalyzedExprKind::Variable(name) = left.expr {
            assert_eq!(name, "nom");
        } else {
            panic!("Expected Variable on left");
        }
        if let AnalyzedExprKind::NumberLiteral(n) = right.expr {
            assert_eq!(n, 1);
        } else {
            panic!("Expected NumberLiteral on right");
        }
    } else {
        panic!("Expected BinOp");
    }
}

#[test]
fn test_extract_binary_op_subject_and_literal() {
    let mut scope = Scope::new();
    scope.define("subj", GlossaType::Number);

    // subject (subj) + literal (1) -> op (+)
    let mut asm = AssembledStatement {
        subject: Some(Constituent {
            lemma: "subj".into(),
            original: "subj".into(),
            normalized: "subj".into(),
            case: Case::Nominative,
            number: Some(Number::Singular),
            gender: None,
            person: None,
        }),
        ..Default::default()
    };
    asm.literals.push(Literal::Number(1));
    asm.operators.push(BinaryOp::Add);

    let result = extract_binary_op(&asm, &scope);
    assert!(result.is_ok());
    let extracted = result.unwrap();
    assert!(extracted.is_some());
    let (expr, ty) = extracted.unwrap();
    assert_eq!(ty, GlossaType::Number);

    if let AnalyzedExprKind::BinOp { left, op, right } = expr.expr {
        assert_eq!(op, BinaryOp::Add);
        if let AnalyzedExprKind::Variable(name) = left.expr {
            assert_eq!(name, "subj");
        } else {
            panic!("Expected Variable on left");
        }
        if let AnalyzedExprKind::NumberLiteral(n) = right.expr {
            assert_eq!(n, 1);
        } else {
            panic!("Expected NumberLiteral on right");
        }
    } else {
        panic!("Expected BinOp");
    }
}

#[test]
fn test_extract_unary_op_negation() {
    let mut scope = Scope::new();
    scope.define("x", GlossaType::Boolean);

    // Unary op (Not) + Literal (true)
    let asm_stmt = AssembledStatement {
        literals: vec![Literal::Boolean(true)],
        unary_operators: vec![UnaryOp::Not],
        ..Default::default()
    };

    let (analyzed, ty) = extract_value(&asm_stmt, &scope).expect("Should extract unary op");

    assert_eq!(ty, GlossaType::Boolean);
    if let AnalyzedExprKind::UnaryOp { op, operand } = analyzed.expr {
        assert_eq!(op, UnaryOp::Not);
        if let AnalyzedExprKind::BooleanLiteral(b) = operand.expr {
            assert!(b);
        } else {
            panic!("Expected boolean literal operand");
        }
    } else {
        panic!("Expected UnaryOp expression");
    }
}

#[test]
fn test_extract_nested_unary_op() {
    let mut scope = Scope::new();
    scope.define("x", GlossaType::Boolean);

    // not not true
    let asm_stmt = AssembledStatement {
        literals: vec![Literal::Boolean(true)],
        unary_operators: vec![UnaryOp::Not, UnaryOp::Not],
        ..Default::default()
    };

    let (analyzed, ty) = extract_value(&asm_stmt, &scope).expect("Should extract nested unary op");

    assert_eq!(ty, GlossaType::Boolean);
    // Outer UnaryOp
    if let AnalyzedExprKind::UnaryOp { op, operand } = analyzed.expr {
        assert_eq!(op, UnaryOp::Not);
        // Inner UnaryOp
        if let AnalyzedExprKind::UnaryOp {
            op: inner_op,
            operand: inner_operand,
        } = operand.expr
        {
            assert_eq!(inner_op, UnaryOp::Not);
            // Innermost Literal
            if let AnalyzedExprKind::BooleanLiteral(b) = inner_operand.expr {
                assert!(b);
            } else {
                panic!("Expected boolean literal operand");
            }
        } else {
            panic!("Expected inner UnaryOp expression");
        }
    } else {
        panic!("Expected outer UnaryOp expression");
    }
}

#[test]
fn test_unary_op_availability() {
    use crate::morphology::lexicon::UnaryOp;
    let _ = UnaryOp::Not;
}
