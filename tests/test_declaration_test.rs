//! Test that δοκιμή test declarations are parsed correctly

use glossa::parser::parse;
use glossa::ast::Statement;

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

    assert_eq!(program.statements.len(), 1, "Expected 1 statement, got {}", program.statements.len());

    // Check that it's a TestDeclaration
    match &program.statements[0] {
        Statement::TestDeclaration(test_decl) => {
            assert_eq!(test_decl.name, "my test");
            assert_eq!(test_decl.body.len(), 0); // Empty body in this test
        }
        other => panic!("Expected TestDeclaration, got {:?}", std::mem::discriminant(other)),
    }
}
