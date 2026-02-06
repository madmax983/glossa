use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

/// Helper to compile GLOSSA source to Rust code
fn compile(source: &str) -> String {
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    generate_rust(&analyzed)
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
    // σημεῖον -> Shmeion (η -> h)
    assert!(code.contains("Shmeion") || code.contains("shmeion"));
    assert!(code.contains("5"));
}

#[test]
fn test_instantiation_multiple_fields() {
    let source = r#"
        εἶδος σημεῖον ὁρίζειν { ξ ἀριθμοῦ· ψ ἀριθμοῦ. }.
        π νέον σημεῖον πέντε τρία ἔστω.
    "#;
    let code = compile(source);
    assert!(code.contains("Shmeion") || code.contains("shmeion"));
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
    // ξ -> x
    assert!(code.contains(".x") || code.contains(". x"));
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
    // ξ -> x
    assert!(code.contains(". x") || code.contains(".x"));
    // ψ -> _u3c8_
    assert!(code.contains(". _u3c8_") || code.contains("._u3c8_"));
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
    // Χρήστης -> _u3c7_rhsths (chi -> _u3c7_, eta -> h)
    assert!(code.contains("struct _u3c7_rhsths"));
    assert!(code.contains("let _u3c7_rhsths = _u3c7_rhsths"));
    assert!(code.contains("Σωκράτης"));
    assert!(code.contains("70"));
}

#[test]
fn test_instantiation_with_boolean_error() {
    let source = r#"
        εἶδος Δοκιμή ὁρίζειν { τιμή ἀριθμοῦ. }.
        δ νέον Δοκιμή ἀληθές ἔστω.
    "#;
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Type mismatch") || err.contains("Τύπος ἀσυμβίβαστος"));
}

#[test]
fn test_instantiation_with_word_number() {
    let source = r#"
        εἶδος Σημεῖον ὁρίζειν { ξ ἀριθμοῦ. }.
        // "πέντε" is a word, but "5" is a number literal?
        // Wait, "πέντε" parses as Expr::Word("πέντε").
        // "5" parses as Expr::NumberLiteral(5) in the parser?
        // Let's check parser. But patterns.rs handles Expr::Word that parses as i64 OR lookups in lexicon.

        // This tests the `Expr::Word` -> `lexicon::numeral_value` path.
        σ νέον Σημεῖον πέντε ἔστω.
    "#;
    let code = compile(source);
    assert!(code.contains("5"));
}

#[test]
fn test_instantiation_with_explicit_numeric_word_error() {
    let source = r#"
        εἶδος Δοκιμή ὁρίζειν { τιμή ἀριθμοῦ. }.
        δ νέον Δοκιμή «42» ἔστω.
    "#;
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Type mismatch") || err.contains("Τύπος ἀσυμβίβαστος"));
}
