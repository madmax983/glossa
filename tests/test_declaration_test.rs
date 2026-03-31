#![allow(missing_docs)]
//! Test that δοκιμή test declarations are parsed correctly

use glossa::ast::Statement;
use glossa::parser::parse;

#[test]
fn test_parse_test_declaration() {
    let source = r#"
δοκιμή «my test».
τέλος.
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    let program = result.unwrap();

    // Debug: print what we got
    for (i, stmt) in program.statements.iter().enumerate() {
        eprintln!("Statement {}: {:?}", i, std::mem::discriminant(stmt));
    }

    assert_eq!(
        program.statements.len(),
        1,
        "Expected 1 statement, got {}",
        program.statements.len()
    );

    // Check that it's a TestDeclaration
    match &program.statements[0] {
        Statement::TestDeclaration(test_decl) => {
            assert_eq!(test_decl.name, "my test");
            assert_eq!(test_decl.body.len(), 0); // Empty body in this test
        }
        other => panic!(
            "Expected TestDeclaration, got {:?}",
            std::mem::discriminant(other)
        ),
    }
}

#[test]
fn test_parse_test_with_body() {
    let source = r#"
δοκιμή «test with statements».
    ξ 5 ἔστω.
    ξ λέγε.
τέλος.
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::TestDeclaration(test_decl) => {
            assert_eq!(test_decl.name, "test with statements");
            assert_eq!(test_decl.body.len(), 2, "Expected 2 body statements");
        }
        other => panic!(
            "Expected TestDeclaration, got {:?}",
            std::mem::discriminant(other)
        ),
    }
}

#[test]
fn test_parse_multiple_tests() {
    let source = r#"
δοκιμή «first test».
    ξ 5 ἔστω.
τέλος.

δοκιμή «second test».
    ψ 10 ἔστω.
τέλος.
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    let program = result.unwrap();
    assert_eq!(program.statements.len(), 2, "Expected 2 test declarations");

    match &program.statements[0] {
        Statement::TestDeclaration(test_decl) => {
            assert_eq!(test_decl.name, "first test");
        }
        _ => panic!("Expected first TestDeclaration"),
    }

    match &program.statements[1] {
        Statement::TestDeclaration(test_decl) => {
            assert_eq!(test_decl.name, "second test");
        }
        _ => panic!("Expected second TestDeclaration"),
    }
}

#[test]
fn test_parse_test_unaccented_keywords() {
    let source = r#"
δοκιμη «unaccented test».
    ξ 5 ἔστω.
τελος.
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Parse failed with unaccented keywords");

    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::TestDeclaration(test_decl) => {
            assert_eq!(test_decl.name, "unaccented test");
            assert_eq!(test_decl.body.len(), 1);
        }
        _ => panic!("Expected TestDeclaration"),
    }
}
