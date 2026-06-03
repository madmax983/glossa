#[test]
fn test_run_scholar_success() {
    let mut temp_file = Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. ἡλικία ἀριθμοῦ. }.

    χαρακτήρ Εὐγενής ὁρίζειν { δεῖ show τῷ self. }.

    χαιρετισμός ὁρίζειν· «χαῖρε» λέγε.

    «γεια» λέγε.";
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let result = glossa::tools::scholar::run_scholar(temp_file.path());
    assert!(result.is_ok(), "Scholar failed: {:?}", result.err());

    let output_path = temp_file.path().with_extension("doc.md");
    assert!(output_path.exists());

    let mut f = std::fs::File::open(&output_path).unwrap();
    let mut md = String::new();
    std::io::Read::take(&mut f, 1024 * 1024 + 1)
        .read_to_string(&mut md)
        .unwrap();

    assert!(md.contains("# API Documentation"));
    assert!(md.contains("## Types (Εἴδη)"));
    assert!(md.contains("### `χρηστης`"));
    assert!(md.contains("## Traits (Χαρακτῆρες)"));
    assert!(md.contains("### `ευγενης`"));
    assert!(md.contains("## Functions (Ἔργα)"));
    assert!(md.contains("### `χαιρετισμος() -> Οὐδέν`"));
}
