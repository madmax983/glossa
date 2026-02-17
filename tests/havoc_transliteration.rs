use glossa::tools::runner::run_file;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_type_identifier_collision() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("collision.gl");

    // "x" (ASCII) and "Ξ" (Greek Xi) both transliterate to "x" -> "g_x" -> "G_x" (Capitalized for struct).
    // This causes a name collision in the generated Rust code (two structs named G_x).
    let code = r#"
        εἶδος x ὁρίζειν { }.
        εἶδος Ξ ὁρίζειν { }.
    "#;

    let mut f = File::create(&file_path).unwrap();
    f.write_all(code.as_bytes()).unwrap();

    // Debug: check generated Rust code
    use glossa::codegen::generate_rust_file;
    use glossa::parser::parse;
    use glossa::semantic::analyze_program;
    let ast = parse(code).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    let rust = generate_rust_file(&analyzed);
    println!("Generated Rust:\n{}", rust);

    let result = run_file(&file_path);

    // We expect this to SUCCEED now, because 'x' maps to 'G_x' and 'Ξ' maps to 'G__u3be_' (hex encoded).
    // No collision occurs.
    assert!(result.is_ok(), "Should succeed as identifier collision is fixed");
}
