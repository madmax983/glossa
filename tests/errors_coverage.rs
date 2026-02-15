use glossa::errors::{
    AssemblyError, GlossaError, case_mismatch, gender_mismatch, help, immutable_assignment,
    number_mismatch, undefined_variable,
};
use glossa::morphology::{Case, Gender, Number, Person};
use miette::SourceSpan;

#[test]
fn test_assembly_error_coverage() {
    // Instantiate every AssemblyError variant
    let double_subject = AssemblyError::DoubleSubject;
    let double_object = AssemblyError::DoubleObject;
    let double_indirect = AssemblyError::DoubleIndirect;
    let double_verb = AssemblyError::DoubleVerb;
    let missing_verb = AssemblyError::MissingVerb;
    let sv_disagreement = AssemblyError::SubjectVerbDisagreement {
        subject: (Some(Person::First), Some(Number::Singular)),
        verb: (Some(Person::Third), Some(Number::Plural)),
    };
    let gender_mismatch = AssemblyError::GenderMismatch {
        word1: "a".into(),
        gender1: Gender::Masculine,
        word2: "b".into(),
        gender2: Gender::Feminine,
    };
    let limit_exceeded = AssemblyError::LimitExceeded {
        resource: "test".into(),
        max: 10,
    };

    // Verify Display impl via to_string()
    assert!(double_subject.to_string().contains("Διπλοῦν ὑποκείμενον"));
    assert!(double_object.to_string().contains("Διπλοῦν ἀντικείμενον"));
    assert!(double_indirect.to_string().contains("Διπλοῦν ἔμμεσον"));
    assert!(double_verb.to_string().contains("Διπλοῦν ῥῆμα"));
    assert!(missing_verb.to_string().contains("Ῥῆμα οὐχ εὑρέθη"));
    assert!(sv_disagreement.to_string().contains("Ἀσυμφωνία"));
    assert!(gender_mismatch.to_string().contains("Ἀσυμφωνία γένους"));
    assert!(limit_exceeded.to_string().contains("Ὑπέρβασις ὁρίου"));

    // Verify Debug impl
    let _ = format!("{:?}", double_subject);
}

#[test]
fn test_messages_coverage() {
    let undefined = undefined_variable("foo");
    assert!(undefined.contains("Οὐκ οἶδα τὸ ὄνομα «foo»"));

    let immutable = immutable_assignment("bar");
    assert!(immutable.contains("Τὸ «bar» ἀμετάβλητόν ἐστιν"));

    let gender = gender_mismatch("foo", Gender::Masculine, "bar", Gender::Feminine);
    assert!(gender.contains("foo"));
    assert!(gender.contains("bar"));

    let number = number_mismatch("foo", Number::Singular, "bar", Number::Plural);
    assert!(number.contains("foo"));
    assert!(number.contains("bar"));

    let case = case_mismatch("foo", Case::Nominative, "bar", Case::Accusative);
    assert!(case.contains("foo"));
    assert!(case.contains("bar"));

    // Help messages
    assert!(!help::BINDING.is_empty());
    assert!(!help::PRINT.is_empty());
    assert!(!help::CASES.is_empty());
}

#[test]
fn test_glossa_error_coverage() {
    // Constructors
    let parse = GlossaError::parse("parse error");
    let parse_source = GlossaError::parse_with_source(
        "parse error",
        "src",
        SourceSpan::new(0usize.into(), 0usize),
    );
    let semantic = GlossaError::semantic("semantic error");
    let undefined = GlossaError::undefined("undefined");
    let agreement = GlossaError::agreement("agreement error");
    let codegen = GlossaError::codegen("codegen error");
    let limit = GlossaError::LimitExceeded {
        resource: "limit".into(),
        max: 100,
    };
    let assembly: GlossaError = AssemblyError::MissingVerb.into();

    // Verify categories
    assert_eq!(parse.category_greek(), "Σύνταξις");
    assert_eq!(parse_source.category_greek(), "Σύνταξις");
    assert_eq!(semantic.category_greek(), "Σημασία");
    assert_eq!(undefined.category_greek(), "Ὄνομα");
    assert_eq!(agreement.category_greek(), "Συμφωνία");
    assert_eq!(codegen.category_greek(), "Κῶδιξ");
    assert_eq!(limit.category_greek(), "Όριον");
    assert_eq!(assembly.category_greek(), "Συναρμογή");

    // Verify Display impl via to_string()
    assert!(parse.to_string().contains("Σφάλμα συντάξεως"));
    assert!(parse_source.to_string().contains("Σφάλμα συντάξεως"));
    assert!(semantic.to_string().contains("Σφάλμα σημασίας"));
    assert!(undefined.to_string().contains("Ἄγνωστον ὄνομα"));
    assert!(agreement.to_string().contains("Σφάλμα συμφωνίας"));
    assert!(codegen.to_string().contains("Σφάλμα κώδικος"));
    assert!(limit.to_string().contains("Ὑπέρβασις ὀρίου"));
    assert!(assembly.to_string().contains("Ῥῆμα οὐχ εὑρέθη")); // AssemblyError inner

    // Verify Debug impl
    let _ = format!("{:?}", parse);
}
