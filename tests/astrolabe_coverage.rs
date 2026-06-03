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
