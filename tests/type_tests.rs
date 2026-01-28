use glossa::ast::build_ast;
use glossa::codegen::generate_rust;
use glossa::ir::lower_to_hir;
use glossa::semantic::analyze_program;

/// Helper to compile GLOSSA source to Rust code
fn compile(source: &str) -> String {
    let ast = build_ast(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    let hir = lower_to_hir(&analyzed);
    generate_rust(&hir)
}

// Cycle 1: Type Declaration Parsing
#[test]
fn test_parse_empty_type() {
    let ast = build_ast("εἶδος μονάς ὁρίζειν { }.").unwrap();
    assert_eq!(ast.statements.len(), 1);
}

#[test]
fn test_parse_type_with_field() {
    let ast = build_ast("εἶδος σημεῖον ὁρίζειν { ξ ἀριθμοῦ. }.").unwrap();
    assert_eq!(ast.statements.len(), 1);
}

// Cycle 2: Type Semantic Analysis
#[test]
fn test_analyze_type_definition() {
    let code = compile("εἶδος σημεῖον ὁρίζειν { ξ ἀριθμοῦ. }.");
    assert!(code.contains("struct"));
}

#[test]
fn test_analyze_type_with_multiple_fields() {
    let code = compile("εἶδος σημεῖον ὁρίζειν { ξ ἀριθμοῦ· ψ ἀριθμοῦ. }.");
    assert!(code.contains("struct"));
}

// Cycle 4: Type Instantiation
#[test]
fn test_instantiation() {
    let source = r#"
        εἶδος σημεῖον ὁρίζειν { ξ ἀριθμοῦ. }.
        π νέον σημεῖον πέντε ἔστω.
    "#;
    let code = compile(source);
    assert!(code.contains("Semeion") || code.contains("semeion"));
    assert!(code.contains("5"));
}

#[test]
fn test_instantiation_multiple_fields() {
    let source = r#"
        εἶδος σημεῖον ὁρίζειν { ξ ἀριθμοῦ· ψ ἀριθμοῦ. }.
        π νέον σημεῖον πέντε τρία ἔστω.
    "#;
    let code = compile(source);
    assert!(code.contains("Semeion") || code.contains("semeion"));
}

// Cycle 5: Field Access
#[test]
fn test_field_access() {
    let source = r#"
        εἶδος σημεῖον ὁρίζειν { ξ ἀριθμοῦ. }.
        π νέον σημεῖον πέντε ἔστω.
        που ξ λέγε.
    "#;
    let code = compile(source);
    eprintln!("Generated code:\n{}", code);
    assert!(code.contains(".xi") || code.contains(". xi"));
}

#[test]
fn test_field_access_multiple_fields() {
    let source = r#"
        εἶδος σημεῖον ὁρίζειν { ξ ἀριθμοῦ· ψ ἀριθμοῦ. }.
        α νέον σημεῖον πέντε τρία ἔστω.
        αου ξ λέγε.
        αου ψ λέγε.
    "#;
    let code = compile(source);
    eprintln!("Generated code:\n{}", code);
    assert!(code.contains(". xi") || code.contains(".xi"));
    assert!(code.contains(". psi") || code.contains(".psi"));
}
