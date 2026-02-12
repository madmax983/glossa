use glossa::ast::{Expr, Statement};
use glossa::parser::builder::parse_source;

#[test]
fn test_greek_numerals_assignment() {
    // x = 22 (κβʹ)
    let source = "ξ κβʹ ἔστω.";
    let program = parse_source(source).unwrap();

    assert_eq!(program.statements.len(), 1);

    let Statement::Regular { clauses, .. } = &program.statements[0] else {
        panic!("Expected Regular Statement");
    };

    let Expr::Phrase(terms) = &clauses[0].expressions[0] else {
        panic!("Expected Phrase");
    };

    // ξ, 22, ἔστω
    assert_eq!(terms.len(), 3);
    let Expr::NumberLiteral(val) = terms[1] else {
        panic!("Expected NumberLiteral, got {:?}", terms[1]);
    };
    assert_eq!(val, 22);
}

#[test]
fn test_greek_numerals_array() {
    // [1, 2, 3] = [αʹ, βʹ, γʹ]
    let source = "[αʹ, βʹ, γʹ] λέγε.";
    let program = parse_source(source).unwrap();

    let Statement::Regular { clauses, .. } = &program.statements[0] else {
        panic!("Expected Regular Statement");
    };

    let Expr::Phrase(terms) = &clauses[0].expressions[0] else {
        panic!("Expected Phrase");
    };

    // [1, 2, 3], λέγε
    let Expr::ArrayLiteral(elements) = &terms[0] else {
        panic!("Expected ArrayLiteral");
    };

    assert_eq!(elements.len(), 3);
    assert!(matches!(elements[0], Expr::NumberLiteral(1)));
    assert!(matches!(elements[1], Expr::NumberLiteral(2)));
    assert!(matches!(elements[2], Expr::NumberLiteral(3)));
}

#[test]
fn test_greek_numerals_index() {
    // arr[10] = arr[ιʹ]
    let source = "πίνακας[ιʹ] λέγε.";
    let program = parse_source(source).unwrap();

    let Statement::Regular { clauses, .. } = &program.statements[0] else {
        panic!("Expected Regular Statement");
    };

    let Expr::Phrase(terms) = &clauses[0].expressions[0] else {
        panic!("Expected Phrase");
    };

    // πίνακας[10], λέγε
    let Expr::IndexAccess { index, .. } = &terms[0] else {
        panic!("Expected IndexAccess");
    };

    let Expr::NumberLiteral(idx) = **index else {
        panic!("Expected NumberLiteral index");
    };
    assert_eq!(idx, 10);
}

#[test]
fn test_greek_numerals_mixed() {
    // 2024 (͵βκδʹ)
    let source = "͵βκδʹ λέγε.";
    let program = parse_source(source).unwrap();

    let Statement::Regular { clauses, .. } = &program.statements[0] else {
        panic!("Expected Regular Statement");
    };

    let Expr::Phrase(terms) = &clauses[0].expressions[0] else {
        panic!("Expected Phrase");
    };

    let Expr::NumberLiteral(val) = terms[0] else {
        panic!("Expected NumberLiteral");
    };
    assert_eq!(val, 2024);
}

#[test]
fn test_arabic_fallback() {
    // 42 λέγε.
    let source = "42 λέγε.";
    let program = parse_source(source).unwrap();

    let Statement::Regular { clauses, .. } = &program.statements[0] else {
        panic!("Expected Regular Statement");
    };

    let Expr::Phrase(terms) = &clauses[0].expressions[0] else {
        panic!("Expected Phrase");
    };

    let Expr::NumberLiteral(val) = terms[0] else {
        panic!("Expected NumberLiteral");
    };
    assert_eq!(val, 42);
}

#[test]
fn test_invalid_greek_logic() {
    // \u{0300}ʹ is a valid Greek char sequence in grammar (Combining Grave + Keraia)
    // But parse_greek_numeral returns error ("Empty or invalid numeral")
    // This tests the map_err path in parse_number_literal
    let source = "\u{0300}ʹ λέγε.";
    let result = parse_source(source);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{}", err).contains("Invalid number"));
}
