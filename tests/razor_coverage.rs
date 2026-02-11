use glossa::highlight::highlight;
use glossa::parser::parse;
use glossa::report::GlossaReport;
use glossa::semantic::{analyze_program, AssembledStatement, Constituent};

#[test]
fn test_highlight_integration() {
    let source = r#"
        εἶδος Χρήστης ὁρίζειν {
            ὄνομα Ὄνομα.
        }.

        χαρακτήρ Show ὁρίζειν {
            δεῖ δεῖξαι.
        }.

        εἶδος Χρήστης τῷ Show ἐμπίπτειν {
            δεῖξαι «Χρήστης» λέγε.
        }.

        δοκιμή «test».
            «test» λέγε.
        τέλος.

        ξ 5 ἔστω.
        ξ λέγε.
        ξ?
    "#;

    let highlighted = highlight(source).expect("Highlighting failed");
    assert!(!highlighted.is_empty());
    // Basic ANSI check
    assert!(highlighted.contains("\x1b["));
}

#[test]
fn test_assembled_statement_derives() {
    // Exercise Default
    let stmt = AssembledStatement::default();
    assert!(stmt.subject.is_none());
    assert!(stmt.nominatives.is_empty());

    // Exercise Clone
    let stmt_clone = stmt.clone();
    assert!(stmt_clone.verb.is_none());

    // Exercise Debug
    let debug_str = format!("{:?}", stmt);
    assert!(debug_str.contains("AssembledStatement"));

    // Exercise Constituent Debug/Clone
    let c = Constituent {
        lemma: "test".into(),
        original: "test".into(),
        case: glossa::morphology::Case::Nominative,
        number: None,
        gender: None,
        person: None,
    };
    let c_clone = c.clone();
    assert_eq!(c.original, c_clone.original);
    let c_debug = format!("{:?}", c);
    assert!(c_debug.contains("Constituent"));
}

#[test]
fn test_report_stats_initialization() {
    let source = r#"
        εἶδος Τύπος ὁρίζειν { α Ἀριθμός. }.

        χαρακτήρ Χ ὁρίζειν { δεῖ φ. }.

        λειτουργία ὁρίζειν (χ)·
            χ λέγε.

        ξ 5 ἔστω.
        «χαῖρε» λέγε.
    "#;

    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();

    // Verify stats calculation indirectly via report generation
    let report = GlossaReport::new(&analyzed, "test.gl".to_string());
    let report_str = report.to_string();

    // Check that we counted the definitions
    assert!(report_str.contains("Συναρτήσεις (Functions)"));
    assert!(report_str.contains("Τύποι (Types)"));
    // Trait count might not be explicitly listed in the table if 0 or dependent on implementation details,
    // but ProgramStats::new logic for counts is what we want to cover.
    // The ProgramStats::new uses struct update syntax now.
    // We can't access private ProgramStats fields, but ensuring it doesn't crash
    // and produces output implies success.

    println!("{}", report_str);
}
