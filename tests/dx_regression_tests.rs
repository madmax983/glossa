use glossa::ast::build_ast;
use glossa::codegen::generate_rust;
use glossa::ir::lower_to_hir;
use glossa::semantic::analyze_program;

fn compile(source: &str) -> String {
    let ast = build_ast(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    let hir = lower_to_hir(&analyzed);
    generate_rust(&hir)
}

#[test]
fn test_readme_hero_example() {
    let source = r#"
        // Define a type (struct) using nominative types (DX fix)
        εἶδος Χρήστης ὁρίζειν {
            ὄνομα ὄνομα.      // field: String
            ἡλικία ἀριθμός.   // field: i64
        }.

        // Create a new user instance using literals (DX fix)
        χρήστης νέον Χρήστης
            «Σωκράτης»
            70
        ἔστω.

        // Access property using genitive form (Lexicon fix)
        χρήστου ὄνομα λέγε.
    "#;

    let code = compile(source);
    println!("Generated code:\n{}", code);

    // Verify nominative type resolution
    assert!(code.contains("struct Chrestes"));
    // quote! puts spaces around colons
    assert!(code.contains("onoma : String"));
    assert!(code.contains("elikia : i64"));

    // Verify literal instantiation and string conversion
    assert!(code.contains("Chrestes {"));
    assert!(code.contains("onoma : \"Σωκράτης\" . to_string ()"));
    assert!(code.contains("elikia : 70"));

    // Verify genitive access
    assert!(code.contains("println ! (\"{}\" , chrestes . onoma)"));
}
