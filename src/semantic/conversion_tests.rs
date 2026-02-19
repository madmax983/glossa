use super::conversion::{extract_value};
use crate::ast::Expr;
use crate::semantic::{
    AnalyzedExprKind, AssembledStatement, Constituent, GlossaType, Literal, Scope,
};
use crate::morphology::Case;

fn make_constituent(original: &str, lemma: &str) -> Constituent {
    Constituent {
        lemma: lemma.into(),
        original: original.into(),
        case: Case::Nominative,
        number: None,
        gender: None,
        person: None,
    }
}

#[test]
fn test_extract_unwrap() {
    let mut scope = Scope::new();
    let mut asm_stmt = AssembledStatement::default();

    // Simulate `x!` where x is a variable
    let expr = Expr::Word(crate::ast::Word {
        original: "x".into(),
        normalized: "x".into(),
    });
    asm_stmt.unwraps.push(expr);

    // Define x in scope so analyze_argument_expr works
    scope.define("x", GlossaType::Option(Box::new(GlossaType::Number)));

    let (analyzed, _) = extract_value(&asm_stmt, &scope).expect("Should extract unwrap");

    assert!(matches!(analyzed.expr, AnalyzedExprKind::Unwrap(_)));
}

#[test]
fn test_extract_subject_none() {
    let scope = Scope::new();
    let mut asm_stmt = AssembledStatement::default();

    // Subject "οὐδέν" (None)
    asm_stmt.subject = Some(make_constituent("οὐδέν", "ουδεν"));

    let (analyzed, glossa_type) = extract_value(&asm_stmt, &scope).expect("Should extract None");

    assert!(matches!(analyzed.expr, AnalyzedExprKind::None));
    assert!(matches!(glossa_type, GlossaType::Option(_)));
}

#[test]
fn test_extract_subject_some() {
    let scope = Scope::new();
    let mut asm_stmt = AssembledStatement::default();

    // Subject "τί" (Some)
    asm_stmt.subject = Some(make_constituent("τί", "τι"));

    // Literal value for Some(val)
    asm_stmt.literals.push(Literal::Number(42));

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
    let mut asm_stmt = AssembledStatement::default();

    asm_stmt.literals.push(Literal::Number(123));

    let (analyzed, glossa_type) = extract_value(&asm_stmt, &scope).expect("Should extract literal");

    if let AnalyzedExprKind::NumberLiteral(n) = analyzed.expr {
        assert_eq!(n, 123);
    } else {
        panic!("Expected NumberLiteral");
    }

    assert_eq!(glossa_type, GlossaType::Number);
}

#[test]
fn test_extract_object_variable() {
    let mut scope = Scope::new();
    let mut asm_stmt = AssembledStatement::default();

    scope.define("foo", GlossaType::String);
    asm_stmt.object = Some(make_constituent("foo", "foo"));

    let (analyzed, _) = extract_value(&asm_stmt, &scope).expect("Should extract variable from object");

    if let AnalyzedExprKind::Variable(name) = analyzed.expr {
        assert_eq!(name, "foo");
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_extract_object_none() {
    let scope = Scope::new();
    let mut asm_stmt = AssembledStatement::default();

    // Object "οὐδέν" (None) - checking if logic handles object as enum variant
    asm_stmt.object = Some(make_constituent("οὐδέν", "ουδεν"));

    let (analyzed, _) = extract_value(&asm_stmt, &scope).expect("Should extract None from object");

    assert!(matches!(analyzed.expr, AnalyzedExprKind::None));
}

#[test]
fn test_extract_object_some() {
    let scope = Scope::new();
    let mut asm_stmt = AssembledStatement::default();

    // Object "τί" (Some)
    asm_stmt.object = Some(make_constituent("τί", "τι"));
    asm_stmt.literals.push(Literal::Number(10));

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
    let mut asm_stmt = AssembledStatement::default();

    // Object "πέντε" (five)
    asm_stmt.object = Some(make_constituent("πέντε", "πεντε"));

    let (analyzed, glossa_type) = extract_value(&asm_stmt, &scope).expect("Should extract numeral from object");

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

    let (analyzed, glossa_type) = extract_value(&asm_stmt, &scope).expect("Should fallback to default");

    if let AnalyzedExprKind::NumberLiteral(n) = analyzed.expr {
        assert_eq!(n, 0);
    } else {
        panic!("Expected default 0");
    }

    assert_eq!(glossa_type, GlossaType::Number);
}
