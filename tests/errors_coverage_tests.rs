use glossa::errors::{
    AssemblyError, GlossaError, case_mismatch, case_name, gender_mismatch, gender_name,
    immutable_assignment, number_mismatch, number_name, type_mismatch, undefined_variable,
};
use glossa::morphology::{Case, Gender, Number, Person};

#[test]
fn test_assembly_error_messages() {
    let err = AssemblyError::DoubleSubject;
    assert!(err.to_string().contains("Διπλοῦν ὑποκείμενον"));

    let err = AssemblyError::DoubleObject;
    assert!(err.to_string().contains("Διπλοῦν ἀντικείμενον"));

    let err = AssemblyError::DoubleIndirect;
    assert!(err.to_string().contains("Διπλοῦν ἔμμεσον"));

    let err = AssemblyError::DoubleVerb;
    assert!(err.to_string().contains("Διπλοῦν ῥῆμα"));

    let err = AssemblyError::MissingVerb;
    assert!(err.to_string().contains("Ῥῆμα οὐχ εὑρέθη"));

    let err = AssemblyError::SubjectVerbDisagreement {
        subject: (Some(Person::First), Some(Number::Singular)),
        verb: (Some(Person::Third), Some(Number::Singular)),
    };
    assert!(err.to_string().contains("Ἀσυμφωνία"));

    let err = AssemblyError::GenderMismatch {
        word1: "word1".to_string(),
        gender1: Gender::Masculine,
        word2: "word2".to_string(),
        gender2: Gender::Feminine,
    };
    assert!(err.to_string().contains("Ἀσυμφωνία γένους"));
}

#[test]
fn test_glossa_error_variants() {
    // Parse
    let err = GlossaError::parse("msg");
    assert_eq!(err.category_greek(), "Σύνταξις");
    assert!(err.to_string().contains("Σφάλμα συντάξεως"));

    // Semantic
    let err = GlossaError::semantic("msg");
    assert_eq!(err.category_greek(), "Σημασία");
    assert!(err.to_string().contains("Σφάλμα σημασίας"));

    // TypeError
    let err = GlossaError::type_error("msg");
    assert_eq!(err.category_greek(), "Τύπος");
    assert!(err.to_string().contains("Σφάλμα τύπου"));

    // Undefined
    let err = GlossaError::undefined("name");
    assert_eq!(err.category_greek(), "Ὄνομα");
    assert!(err.to_string().contains("Ἄγνωστον ὄνομα"));

    // Agreement
    let err = GlossaError::agreement("msg");
    assert_eq!(err.category_greek(), "Συμφωνία");
    assert!(err.to_string().contains("Σφάλμα συμφωνίας"));

    // Codegen
    let err = GlossaError::codegen("msg");
    assert_eq!(err.category_greek(), "Κῶδιξ");
    assert!(err.to_string().contains("Σφάλμα κώδικος"));

    // IO
    let err = GlossaError::io("msg");
    assert_eq!(err.category_greek(), "Ἀρχεῖον");
    assert!(err.to_string().contains("Σφάλμα ἀρχείου"));

    // Assembly (wrapped)
    let asm_err = AssemblyError::DoubleSubject;
    let err = GlossaError::AssemblyError(asm_err);
    assert_eq!(err.category_greek(), "Συναρμογή");
    assert!(err.to_string().contains("Διπλοῦν ὑποκείμενον"));
}

#[test]
fn test_helper_messages() {
    assert!(type_mismatch("A", "B").contains("Ἐδόκει"));
    assert!(undefined_variable("X").contains("Οὐκ οἶδα"));
    assert!(immutable_assignment("X").contains("ἀμετάβλητόν"));

    assert!(gender_mismatch("A", Gender::Masculine, "B", Gender::Feminine).contains("ἀρσενικόν"));
    assert!(number_mismatch("A", Number::Singular, "B", Number::Plural).contains("ἑνικός"));
    assert!(case_mismatch("A", Case::Nominative, "B", Case::Accusative).contains("ὀνομαστική"));
}

#[test]
fn test_enum_names() {
    assert_eq!(gender_name(Gender::Masculine), "ἀρσενικόν");
    assert_eq!(gender_name(Gender::Feminine), "θηλυκόν");
    assert_eq!(gender_name(Gender::Neuter), "οὐδέτερον");

    assert_eq!(number_name(Number::Singular), "ἑνικός");
    assert_eq!(number_name(Number::Plural), "πληθυντικός");

    assert_eq!(case_name(Case::Nominative), "ὀνομαστική");
    assert_eq!(case_name(Case::Genitive), "γενική");
    assert_eq!(case_name(Case::Dative), "δοτική");
    assert_eq!(case_name(Case::Accusative), "αἰτιατική");
    assert_eq!(case_name(Case::Vocative), "κλητική");
}
