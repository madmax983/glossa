use glossa::semantic::GlossaType;
use glossa::tools::merchant::run_merchant;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_run_merchant() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.gl");
    fs::write(&file_path, "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.").unwrap();

    let res = run_merchant(&file_path);
    assert!(res.is_ok());
}

#[test]
fn test_run_merchant_missing_file() {
    let res = run_merchant(&PathBuf::from("does_not_exist.gl"));
    assert!(res.is_err());
}

#[test]
fn test_run_merchant_no_types() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.gl");
    fs::write(&file_path, "1.").unwrap();

    let res = run_merchant(&file_path);
    assert!(res.is_ok());
}

#[test]
fn test_run_merchant_invalid_syntax() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.gl");
    fs::write(&file_path, "not valid greek").unwrap();

    let res = run_merchant(&file_path);
    assert!(res.is_err());
}

#[test]
fn test_merchant_unit_type() {
    let mut out = String::new();
    glossa::tools::merchant::write_glossa_type_to_json_schema(&GlossaType::Unit, &mut out).unwrap();
    assert_eq!(out, r#"{"type": "object"}"#);
}

#[test]
fn test_merchant_function_type() {
    let mut out = String::new();
    let func = GlossaType::Function {
        params: vec![],
        returns: Box::new(GlossaType::Number),
    };
    glossa::tools::merchant::write_glossa_type_to_json_schema(&func, &mut out).unwrap();
    assert_eq!(out, r#"{"type": "object"}"#);
}

#[test]
fn test_merchant_unknown_type() {
    let mut out = String::new();
    glossa::tools::merchant::write_glossa_type_to_json_schema(
        &glossa::semantic::GlossaType::Unknown,
        &mut out,
    )
    .unwrap();
    assert_eq!(out, r#"{"type": "object"}"#);
}
