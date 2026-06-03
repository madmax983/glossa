#[cfg(feature = "nova")]
#[test]
fn test_astrolabe_full_coverage() {
    use glossa::tools::astrolabe::run_astrolabe;
    let _code = "
    ξ «χαῖρε» ἔστω.
    «κόσμε» λέγε.
    εἰ ἀληθές ἐστι, «ναι» λέγε.
    ";

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("astrolabe_test.gl");
    std::fs::write(&file_path, _code).unwrap();

    let res = run_astrolabe(&file_path);
    assert!(res.is_ok());
}

#[cfg(feature = "nova")]
#[test]
fn test_astrolabe_empty_strings() {
    use glossa::tools::astrolabe::run_astrolabe;
    let _code = "
    ξ πέντε ἔστω.
    ξ λέγε.
    ";

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("astrolabe_empty_test.gl");
    std::fs::write(&file_path, _code).unwrap();

    let res = run_astrolabe(&file_path);
    assert!(res.is_ok());
}
#[cfg(feature = "nova")]
#[test]
fn test_astrolabe_extract_expressions() {
    use glossa::tools::astrolabe::run_astrolabe;
    let _code = "
    εἶδος Χρήστης ὁρίζειν {
       ὄνομα ὀνόματος.
    }.
    χρήστης νέον Χρήστης «Σωκράτης» ἔστω.
    «1» «2» ἄθροισμα λέγε.
    [«1», «2»] λέγε.
    ";

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("astrolabe_expressions_test.gl");
    std::fs::write(&file_path, _code).unwrap();

    let res = run_astrolabe(&file_path);
    assert!(res.is_ok());
}
#[cfg(feature = "nova")]
#[test]
fn test_astrolabe_more_expressions() {
    use glossa::tools::astrolabe::run_astrolabe;
    let _code = "
    δοκιμή «test_print».
        «test» λέγε.
    τέλος.
    ";

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("astrolabe_more_test.gl");
    std::fs::write(&file_path, _code).unwrap();

    let res = run_astrolabe(&file_path);
    assert!(res.is_ok());
}

#[cfg(feature = "nova")]
#[test]
fn test_astrolabe_loops_and_match() {
    use glossa::tools::astrolabe::run_astrolabe;
    let _code = "
    ἕως ἀληθές, παῦε.
    ";

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("astrolabe_loops_test.gl");
    std::fs::write(&file_path, _code).unwrap();

    let res = run_astrolabe(&file_path);
    assert!(res.is_ok());
}

#[cfg(feature = "nova")]
#[cfg(feature = "nova")]
#[test]
fn test_astrolabe_match() {
    use glossa::tools::astrolabe::run_astrolabe;
    let _code = "
    x 1 ἔστω.
    ";

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("astrolabe_match_test.gl");
    std::fs::write(&file_path, _code).unwrap();

    let _res = run_astrolabe(&file_path);
}

#[cfg(feature = "nova")]
#[test]
fn test_astrolabe_extract_expressions_full() {
    use glossa::tools::astrolabe::run_astrolabe;
    let _code = r#"
    πρόσθεσις ὁρίζειν τῷ α ἀριθμοῦ τῷ β ἀριθμοῦ · α β ἄθροισμα δός.
    «χαῖρε»!
    «κόσμε»;
    τί «value».
    οὐδέν.
    ἐπιτυχία «success».
    σφάλμα «error».
    [«1», «2»].
    "#;

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("astrolabe_expr_full.gl");
    std::fs::write(&file_path, _code).unwrap();

    let _res = run_astrolabe(&file_path);
}
