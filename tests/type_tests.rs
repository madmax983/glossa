use glossa::codegen::generate_rust;
use glossa::ir::lower_to_hir;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

/// Helper to compile GLOSSA source to Rust code
fn compile(source: &str) -> String {
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    let hir = lower_to_hir(&analyzed);
    generate_rust(&hir)
}

// Cycle 1: Type Declaration Parsing
#[test]
fn test_parse_empty_type() {
    let ast = parse("εἶδος μονάς ὁρίζειν { }.").unwrap();
    assert_eq!(ast.statements.len(), 1);
}

#[test]
fn test_parse_type_with_field() {
    let ast = parse("εἶδος σημεῖον ὁρίζειν { ξ ἀριθμοῦ. }.").unwrap();
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

#[test]
fn test_instantiation_with_literals() {
    let source = r#"
        εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος· ἡλικία ἀριθμοῦ. }.
        χρήστης νέον Χρήστης «Σωκράτης» 70 ἔστω.
    "#;
    let code = compile(source);
    eprintln!("Generated code:\n{}", code);
    // It should generate struct instantiation, not string assignment
    // We expect: let chrestes = Chrestes { onoma: "Σωκράτης".to_string(), elikia: 70 };
    assert!(code.contains("struct Chrestes"));
    assert!(code.contains("let chrestes = Chrestes"));
    assert!(code.contains("Σωκράτης"));
    assert!(code.contains("70"));
}
