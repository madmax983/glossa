use glossa::tools::emissary::run_emissary;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_emissary_coverage() {
    let dir = tempdir().expect("Failed to create tempdir");
    let input_path = dir.path().join("test.gl");

    // File not found test
    let res = run_emissary(Path::new("does_not_exist.gl"));
    assert!(res.is_err());

    // Analysis error test
    fs::write(&input_path, "invalid syntax!!!").expect("Failed to write");
    let res = run_emissary(&input_path);
    assert!(res.is_err());

    // Valid code, no type definition
    fs::write(&input_path, "ξ 10 ἔστω.").expect("Failed to write");
    let res = run_emissary(&input_path);
    assert!(res.is_ok());

    // Valid code with type definition and all field types to hit full code paths
    let valid_code = r#"
    εἶδος Χρήστης ὁρίζειν {
        ὄνομα ὀνόματος.
        ἡλικία ἀριθμοῦ.
    }.
    "#;
    fs::write(&input_path, valid_code).expect("Failed to write");
    let res = run_emissary(&input_path);
    if let Err(ref e) = res {
        println!("Error: {:?}", e);
    }
    assert!(res.is_ok());
}
