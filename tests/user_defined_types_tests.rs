use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

/// Helper to compile GLOSSA source to Rust code
fn compile(source: &str) -> String {
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    generate_rust(&analyzed)
}

#[test]
fn test_function_with_struct_parameter() {
    let source = "
    εἶδος Σημεῖον ὁρίζειν {
        χ ἀριθμοῦ.
        ψ ἀριθμοῦ.
    }.

    απόσταση ὁρίζειν τῷ σ Σημείου·
        δός σ.χ
    .
    ";

    let code = compile(source);
    eprintln!("Generated code:\n{}", code);

    // Should contain the struct definition
    // Σημεῖον -> normalized "σημειον" -> sanitized "shmeion" -> capitalized "Shmeion"
    assert!(code.contains("struct Shmeion"));

    // Should contain the function definition with the correct type
    // Σημείου (genitive) -> Σημεῖον (nominative)
    // The parameter type should be the struct name

    // Note: If type is unknown, it generates "_"
    assert!(
        !code.contains("s: _"),
        "Generated 's: _' which means type was not resolved"
    );

    // We expect the struct name in the function signature
    // Currently generated as value (move), e.g. "s: Shmeion"
    assert!(code.contains("s : Shmeion") || code.contains("s: Shmeion"));
}
