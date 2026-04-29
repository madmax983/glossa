use glossa::tools::diplomat::run_diplomat;
use std::fs;

#[test]
fn test_run_diplomat_full_coverage() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("full_api.γλ");
    let source = r#"
        εἶδος Χρήστης ὁρίζειν {
            ὄνομα ὀνόματος.
            ἡλικία ἀριθμοῦ.
            εἶναι ἀληθοῦς.
        }.

        ἔργον μετάφρασις ὁρίζειν τῷ κειμένῳ ὀνόματος· δός «καλημέρα».
    "#;
    fs::write(&input_path, source).unwrap();

    let result = run_diplomat(&input_path);
    assert!(result.is_ok());

    let output_path = input_path.with_extension("d.ts");
    assert!(output_path.exists());
    let ts_code = fs::read_to_string(&output_path).unwrap();
    assert!(ts_code.contains("export interface χρηστης"));
    assert!(ts_code.contains("ονομα: string;"));
    assert!(ts_code.contains("ηλικια: number;"));
    assert!(ts_code.contains("ειναι: boolean;"));
    assert!(ts_code.contains("export declare function μεταφρασις(arg0: string): string;"));
}

#[test]
fn test_run_diplomat_error() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("error_api.γλ");
    let source = "invalid syntax";
    fs::write(&input_path, source).unwrap();

    let result = run_diplomat(&input_path);
    assert!(result.is_err());
}

#[test]
fn test_run_diplomat_write_error() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("api.γλ");
    fs::write(&input_path, "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.").unwrap();

    // Create a directory at the expected output path so that fs::write fails
    let output_path = input_path.with_extension("d.ts");
    fs::create_dir(&output_path).unwrap();

    let result = run_diplomat(&input_path);
    assert!(result.is_err());
}

#[test]
fn test_run_diplomat_complex_types() {
    use glossa::semantic::GlossaType;
    use glossa::tools::diplomat::translate_to_ts;

    // Test Set
    assert_eq!(
        translate_to_ts(&GlossaType::Set(Box::new(GlossaType::Number))),
        "Set<number>"
    );

    // Test Map
    assert_eq!(
        translate_to_ts(&GlossaType::Map(
            Box::new(GlossaType::String),
            Box::new(GlossaType::Number)
        )),
        "Map<string, number>"
    );

    // Test Result
    assert_eq!(
        translate_to_ts(&GlossaType::Result(
            Box::new(GlossaType::Number),
            Box::new(GlossaType::String)
        )),
        "number | Error<string>"
    );

    // Test Struct
    assert_eq!(
        translate_to_ts(&GlossaType::Struct {
            name: "MyStruct".into(),
            gender: glossa::morphology::Gender::Neuter,
            fields: vec![],
        }),
        "MyStruct"
    );

    // Test Function with multiple params
    assert_eq!(
        translate_to_ts(&GlossaType::Function {
            params: vec![GlossaType::Number, GlossaType::String],
            returns: Box::new(GlossaType::Boolean)
        }),
        "(arg0: number, arg1: string) => boolean"
    );

    // Test Unit
    assert_eq!(translate_to_ts(&GlossaType::Unit), "void");
}
