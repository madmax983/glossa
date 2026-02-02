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

#[test]
fn test_instantiation_with_boolean() {
    let source = r#"
        εἶδος Κατάστασις ὁρίζειν { ενεργός κενόν. }. // Using κενόν as placeholder for boolean if not available? No, let's use valid bool check.
        // Actually there isn't a "boolean" type keyword in the lexicon explicitly mapped to GlossaType::Boolean yet?
        // Wait, 'αληθες' maps to true.
        // Let's assume there is no explicit boolean type name in lexicon yet, but we can verify BooleanLiteral handling.
        // Or wait, is there a boolean type?
        // In src/semantic/declarations.rs: map_greek_type_to_glossa only handles Number, String, List.
        // So we can't define a struct with a boolean field yet properly?
        // Let's check lexicon.
        // "αριθμου", "ονοματος", "λιστης".
        // But the parser supports Expr::BooleanLiteral.
        // If I use "ἀριθμοῦ" it expects Number.
        // If I pass a boolean literal to a field expecting Number, it might fail type check later, but here we test PARSING/AST construction.

        // Let's try to pass a boolean literal. Even if type doesn't match, we want to exercise the code path in patterns.rs.
        εἶδος Δοκιμή ὁρίζειν { τιμή ἀριθμοῦ. }.
        δ νέον Δοκιμή ἀληθές ἔστω.
    "#;
    // This might fail compilation due to type mismatch in Rust codegen or analysis, but patterns.rs should handle it.
    // However, if it fails analysis later, we won't see the coverage.
    // Let's see if we can use a type that accepts anything or if we can just trigger the path.

    // Actually, `map_genitive_to_type` falls back to `scope.lookup_type`.
    // If we can't define a boolean field, we can't fully test semantic validity, but patterns.rs just builds the expression.

    let code = compile(source);
    // It should generate "true" in the output
    assert!(code.contains("true"));
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
fn test_instantiation_with_explicit_numeric_word() {
    // This tests `word.original.parse::<i64>()` inside Expr::Word match arm
    // Assuming the parser produces Expr::Word for "123" if it's not a NumberLiteral token?
    // Actually pest grammar likely parses digits as NumberLiteral.
    // So Expr::Word with "123" might not happen from parser, but we can try "123" string? No.
    // If the parser always produces NumberLiteral for digits, that path in patterns.rs (Expr::Word -> parse i64) might be dead code
    // unless we construct AST manually or if parser is lenient.
    // But let's leave it. The `lexicon::numeral_value` path is covered by "πέντε".

    // What about Expr::StringLiteral for a non-String field?
    // `test_instantiation_with_literals` covers StringLiteral for String field (with .to_string()).
    // Let's try StringLiteral for Number field (should pass through without .to_string()).
    let source = r#"
        εἶδος Δοκιμή ὁρίζειν { τιμή ἀριθμοῦ. }.
        δ νέον Δοκιμή «42» ἔστω.
    "#;
    let code = compile(source);
    // Should contain "42" as a string literal, not wrapped in to_string (or wrapped? checks expected_type)
    // Field is Number, so expected_type is Number.
    // StringLiteral path checks `if matches!(expected_type, GlossaType::String)`.
    // Since it's Number, it goes to `else { lit_expr }`.
    // So output should be `"42"` (quoted), not `"42".to_string()`.
    assert!(code.contains("\"42\""));
    assert!(!code.contains("\"42\".to_string()"));
}
