use glossa::errors::{AssemblyError, GlossaError};
use glossa::morphology::{Case, Gender, Number, Person};
use glossa::semantic::{AssembledStatement, Constituent, GlossaType};
use smol_str::SmolStr;

#[test]
fn test_glossa_error_display() {
    // Test ParseError
    let err = GlossaError::parse("unexpected token");
    assert!(err.to_string().contains("Σφάλμα συντάξεως"));
    assert!(err.to_string().contains("unexpected token"));
    assert_eq!(err.category_greek(), "Σύνταξις");

    // Test SemanticError
    let err = GlossaError::semantic("type mismatch");
    assert!(err.to_string().contains("Σφάλμα σημασίας"));
    assert!(err.to_string().contains("type mismatch"));
    assert_eq!(err.category_greek(), "Σημασία");

    // Test UndefinedName
    let err = GlossaError::undefined("x");
    assert!(err.to_string().contains("Ἄγνωστον ὄνομα"));
    assert!(err.to_string().contains("x"));
    assert_eq!(err.category_greek(), "Ὄνομα");

    // Test AgreementError
    let err = GlossaError::agreement("bad case");
    assert!(err.to_string().contains("Σφάλμα συμφωνίας"));
    assert_eq!(err.category_greek(), "Συμφωνία");

    // Test CodegenError
    let err = GlossaError::codegen("fail");
    assert!(err.to_string().contains("Σφάλμα κώδικος"));
    assert_eq!(err.category_greek(), "Κῶδιξ");

    // Test LimitExceeded
    let err = GlossaError::LimitExceeded {
        resource: "recursion".into(),
        max: 100,
    };
    assert!(err.to_string().contains("Ὑπέρβασις ὀρίου"));
    assert_eq!(err.category_greek(), "Όριον");

    // Test AssemblyError wrapping
    let asm_err = AssemblyError::DoubleSubject;
    let err = GlossaError::from(asm_err);
    assert!(err.to_string().contains("Διπλοῦν ὑποκείμενον"));
    assert_eq!(err.category_greek(), "Συναρμογή");
}

#[test]
fn test_assembly_error_display() {
    // DoubleSubject
    let err = AssemblyError::DoubleSubject;
    assert!(err.to_string().contains("Διπλοῦν ὑποκείμενον"));

    // DoubleObject
    let err = AssemblyError::DoubleObject;
    assert!(err.to_string().contains("Διπλοῦν ἀντικείμενον"));

    // DoubleIndirect
    let err = AssemblyError::DoubleIndirect;
    assert!(err.to_string().contains("Διπλοῦν ἔμμεσον αντικείμενον"));

    // DoubleVerb
    let err = AssemblyError::DoubleVerb;
    assert!(err.to_string().contains("Διπλοῦν ῥῆμα"));

    // MissingVerb
    let err = AssemblyError::MissingVerb;
    assert!(err.to_string().contains("Ῥῆμα οὐχ εὑρέθη"));

    // SubjectVerbDisagreement
    let err = AssemblyError::SubjectVerbDisagreement {
        subject: (Some(Person::First), Some(Number::Singular)),
        verb: (Some(Person::Third), Some(Number::Plural)),
    };
    assert!(err.to_string().contains("Ἀσυμφωνία"));

    // GenderMismatch
    let err = AssemblyError::GenderMismatch {
        word1: "w1".into(),
        gender1: Gender::Masculine,
        word2: "w2".into(),
        gender2: Gender::Feminine,
    };
    assert!(err.to_string().contains("Ἀσυμφωνία γένους"));

    // LimitExceeded
    let err = AssemblyError::LimitExceeded {
        resource: "test".into(),
        max: 5,
    };
    assert!(err.to_string().contains("Ὑπέρβασις ὁρίου"));
}

#[test]
fn test_glossa_type_display_and_properties() {
    let t = GlossaType::Number;
    assert_eq!(t.to_string(), "Ἀριθμός");
    assert_eq!(t.to_greek(), "ἀριθμός");
    assert!(t.is_compatible(&GlossaType::Number));
    assert!(t.is_compatible(&GlossaType::Unknown));

    let t = GlossaType::String;
    assert_eq!(t.to_string(), "Ὄνομα");
    assert_eq!(t.to_greek(), "ὄνομα");

    let t = GlossaType::Boolean;
    assert_eq!(t.to_string(), "Ἀληθές/Ψεῦδος");
    assert_eq!(t.to_greek(), "ἀληθές");

    let t = GlossaType::Unit;
    assert_eq!(t.to_string(), "Οὐδέν");
    assert_eq!(t.to_greek(), "οὐδέν");

    let t = GlossaType::Unknown;
    assert_eq!(t.to_string(), "Ἄγνωστον");
    assert_eq!(t.to_greek(), "ἄγνωστον");

    let t = GlossaType::List(Box::new(GlossaType::Number));
    assert_eq!(t.to_string(), "Λίστη<Ἀριθμός>");
    assert_eq!(t.to_greek(), "λίστη");
    assert!(t.is_compatible(&GlossaType::List(Box::new(GlossaType::Number))));
    assert!(t.is_compatible(&GlossaType::List(Box::new(GlossaType::Unknown))));

    let t = GlossaType::Set(Box::new(GlossaType::Number));
    assert_eq!(t.to_string(), "Σύνολον<Ἀριθμός>");
    assert_eq!(t.to_greek(), "σύνολον");

    let t = GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number));
    assert_eq!(t.to_string(), "Χάρτης<Ὄνομα, Ἀριθμός>");
    assert_eq!(t.to_greek(), "χάρτης");
    assert!(t.is_compatible(&GlossaType::Map(
        Box::new(GlossaType::String),
        Box::new(GlossaType::Unknown)
    )));

    let t = GlossaType::Option(Box::new(GlossaType::Number));
    assert_eq!(t.to_string(), "Εὑρεθείη<Ἀριθμός>");
    assert_eq!(t.to_greek(), "εὑρεθείη");

    let t = GlossaType::Result(Box::new(GlossaType::Unit), Box::new(GlossaType::String));
    assert_eq!(t.to_string(), "Ἀποτέλεσμα<Οὐδέν, Ὄνομα>");
    assert_eq!(t.to_greek(), "ἀποτέλεσμα");

    let t = GlossaType::Struct {
        name: "User".into(),
        gender: Gender::Masculine,
        fields: vec![],
    };
    assert_eq!(t.to_string(), "Εἶδος User");
    assert_eq!(t.to_greek(), "εἶδος");

    let t = GlossaType::Function {
        params: vec![GlossaType::Number],
        returns: Box::new(GlossaType::Boolean),
    };
    assert_eq!(t.to_string(), "Ἔργον(Ἀριθμός) -> Ἀληθές/Ψεῦδος");
    assert_eq!(t.to_greek(), "ἔργον");
}

#[test]
fn test_assembled_statement_defaults() {
    let stmt = AssembledStatement::default();
    assert!(stmt.subject.is_none());
    assert!(stmt.verb.is_none());
    assert!(!stmt.is_query);

    let cloned = stmt.clone();
    assert!(cloned.subject.is_none());
    // Use Debug
    let _ = format!("{:?}", stmt);
}

#[test]
fn test_constituent_debug_clone() {
    let c = Constituent {
        lemma: "lemma".into(),
        original: "original".into(),
        case: Case::Nominative,
        number: Some(Number::Singular),
        gender: Some(Gender::Masculine),
        person: Some(Person::Third),
    };
    let cloned = c.clone();
    assert_eq!(cloned.lemma, "lemma");
    let _ = format!("{:?}", c);
}
