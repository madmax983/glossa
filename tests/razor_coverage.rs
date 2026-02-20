use glossa::errors::{
    AssemblyError, GlossaError, case_mismatch, gender_mismatch, immutable_assignment,
    number_mismatch, undefined_variable,
};
use glossa::morphology::{Case, Gender, Number};
use glossa::semantic::{AssembledStatement, Constituent, GlossaType, detect_collection_type};
use smol_str::SmolStr;

#[test]
fn test_errors_coverage() {
    // Test GlossaError constructors
    let err = GlossaError::parse("parse error");
    assert!(format!("{}", err).contains("Σφάλμα συντάξεως"));
    assert_eq!(err.category_greek(), "Σύνταξις");

    let err = GlossaError::semantic("semantic error");
    assert!(format!("{}", err).contains("Σφάλμα σημασίας"));
    assert_eq!(err.category_greek(), "Σημασία");

    let err = GlossaError::undefined("x");
    assert!(format!("{}", err).contains("Ἄγνωστον ὄνομα"));
    assert_eq!(err.category_greek(), "Ὄνομα");

    let err = GlossaError::agreement("agreement error");
    assert!(format!("{}", err).contains("Σφάλμα συμφωνίας"));
    assert_eq!(err.category_greek(), "Συμφωνία");

    let err = GlossaError::codegen("codegen error");
    assert!(format!("{}", err).contains("Σφάλμα κώδικος"));
    assert_eq!(err.category_greek(), "Κῶδιξ");

    // Test AssemblyError conversion
    let asm_err = AssemblyError::DoubleSubject;
    let err = GlossaError::from(asm_err);
    assert!(format!("{}", err).contains("Διπλοῦν ὑποκείμενον"));
    assert_eq!(err.category_greek(), "Συναρμογή");

    // Test helper messages
    assert!(undefined_variable("x").contains("Οὐκ οἶδα"));
    assert!(immutable_assignment("x").contains("ἀμετάβλητόν"));
    assert!(gender_mismatch("a", Gender::Masculine, "b", Gender::Feminine).contains("οὐ συμφωνεῖ"));
    assert!(number_mismatch("a", Number::Singular, "b", Number::Plural).contains("οὐ συμφωνεῖ"));
    assert!(case_mismatch("a", Case::Nominative, "b", Case::Accusative).contains("οὐ συμφωνεῖ"));
}

#[test]
fn test_assembly_error_variants() {
    let err = AssemblyError::DoubleObject;
    assert!(format!("{}", err).contains("Διπλοῦν ἀντικείμενον"));

    let err = AssemblyError::DoubleIndirect;
    assert!(format!("{}", err).contains("Διπλοῦν ἔμμεσον"));

    let err = AssemblyError::DoubleVerb;
    assert!(format!("{}", err).contains("Διπλοῦν ῥῆμα"));

    let err = AssemblyError::MissingVerb;
    assert!(format!("{}", err).contains("Ῥῆμα οὐχ εὑρέθη"));

    let err = AssemblyError::SubjectVerbDisagreement {
        subject: (None, Some(Number::Singular)),
        verb: (None, Some(Number::Plural)),
    };
    assert!(format!("{}", err).contains("Ἀσυμφωνία"));

    let err = AssemblyError::GenderMismatch {
        word1: "a".into(),
        gender1: Gender::Masculine,
        word2: "b".into(),
        gender2: Gender::Feminine,
    };
    assert!(format!("{}", err).contains("Ἀσυμφωνία γένους"));

    let err = AssemblyError::LimitExceeded {
        resource: "Test".into(),
        max: 10,
    };
    assert!(format!("{}", err).contains("Ὑπέρβασις ὁρίου"));
}

#[test]
fn test_glossa_type_coverage() {
    // Display
    assert_eq!(format!("{}", GlossaType::Number), "Ἀριθμός");
    assert_eq!(format!("{}", GlossaType::String), "Ὄνομα");
    assert_eq!(format!("{}", GlossaType::Boolean), "Ἀληθές/Ψεῦδος");
    assert_eq!(format!("{}", GlossaType::Unit), "Οὐδέν");
    assert_eq!(format!("{}", GlossaType::Unknown), "Ἄγνωστον");

    // To Greek
    assert_eq!(GlossaType::Number.to_greek(), "ἀριθμός");
    assert_eq!(GlossaType::String.to_greek(), "ὄνομα");
    assert_eq!(GlossaType::Boolean.to_greek(), "ἀληθές");
    assert_eq!(GlossaType::Unit.to_greek(), "οὐδέν");
    assert_eq!(GlossaType::Unknown.to_greek(), "ἄγνωστον");
    assert_eq!(
        GlossaType::List(Box::new(GlossaType::Number)).to_greek(),
        "λίστη"
    );
    assert_eq!(
        GlossaType::Set(Box::new(GlossaType::Number)).to_greek(),
        "σύνολον"
    );
    assert_eq!(
        GlossaType::Map(Box::new(GlossaType::Number), Box::new(GlossaType::Number)).to_greek(),
        "χάρτης"
    );
    assert_eq!(
        GlossaType::Option(Box::new(GlossaType::Number)).to_greek(),
        "εὑρεθείη"
    );
    assert_eq!(
        GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String)).to_greek(),
        "ἀποτέλεσμα"
    );
    assert_eq!(
        GlossaType::Struct {
            name: "Test".into(),
            gender: Gender::Neuter,
            fields: vec![]
        }
        .to_greek(),
        "εἶδος"
    );
    assert_eq!(
        GlossaType::Function {
            params: vec![],
            returns: Box::new(GlossaType::Unit)
        }
        .to_greek(),
        "ἔργον"
    );

    // Compatibility
    assert!(GlossaType::Number.is_compatible(&GlossaType::Number));
    assert!(!GlossaType::Number.is_compatible(&GlossaType::String));
    assert!(GlossaType::Unknown.is_compatible(&GlossaType::Number));
    assert!(GlossaType::Number.is_compatible(&GlossaType::Unknown));

    // Nested compatibility
    assert!(
        GlossaType::List(Box::new(GlossaType::Unknown))
            .is_compatible(&GlossaType::List(Box::new(GlossaType::Number)))
    );
    assert!(
        !GlossaType::List(Box::new(GlossaType::String))
            .is_compatible(&GlossaType::List(Box::new(GlossaType::Number)))
    );
}

#[test]
fn test_detect_collection_type_coverage() {
    let (name, ty) = detect_collection_type("συνολον").unwrap();
    assert_eq!(name, "HashSet");
    assert!(matches!(ty, GlossaType::Set(_)));

    let (name, ty) = detect_collection_type("χαρτης").unwrap();
    assert_eq!(name, "HashMap");
    assert!(matches!(ty, GlossaType::Map(_, _)));

    assert!(detect_collection_type("other").is_none());
}

#[test]
fn test_assembler_structs_coverage() {
    let stmt = AssembledStatement::default();
    assert!(stmt.subject.is_none());
    assert!(stmt.nominatives.is_empty());
    assert!(!stmt.is_query);

    let c = Constituent {
        lemma: SmolStr::new("test"),
        original: SmolStr::new("test"),
        case: Case::Nominative,
        number: None,
        gender: None,
        person: None,
    };
    let debug = format!("{:?}", c);
    assert!(debug.contains("test"));
}
