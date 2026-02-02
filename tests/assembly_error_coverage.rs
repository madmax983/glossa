use glossa::errors::GlossaError;
use glossa::errors::AssemblyError;
use glossa::morphology::{Case, Gender, Number, Person, analyze};
use glossa::semantic::assembler::Assembler;

#[test]
fn test_double_object_error() {
    let mut asm = Assembler::new();

    // Feed first object
    let obj1 = analyze("λόγον");
    asm.feed(&obj1, "λόγον").unwrap();

    // Feed second object
    let obj2 = analyze("ἄνθρωπον");
    let result = asm.feed(&obj2, "ἄνθρωπον");

    assert!(matches!(result, Err(AssemblyError::DoubleObject)));
}

#[test]
fn test_subject_verb_disagreement_error() {
    let mut asm = Assembler::new();

    // Subject: I (1st person sg) - needs manual construction as "ego" might not be in lexicon
    use glossa::morphology::{MorphAnalysis, PartOfSpeech};
    let subject = MorphAnalysis {
        lemma: "εγω".into(),
        part_of_speech: PartOfSpeech::Pronoun,
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        gender: None,
        person: Some(Person::First),
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };
    asm.feed(&subject, "ἐγώ").unwrap();

    // Verb: He says (3rd person sg)
    let verb = analyze("λέγει"); // λεγει is 3rd person sg
    asm.feed(&verb, "λέγει").unwrap();

    let result = asm.finalize();
    assert!(matches!(
        result,
        Err(AssemblyError::SubjectVerbDisagreement { .. })
    ));
}

#[test]
fn test_glossa_error_conversion() {
    let asm_err = AssemblyError::DoubleVerb;
    let glossa_err: GlossaError = asm_err.clone().into();

    match &glossa_err {
        GlossaError::AssemblyError(e) => assert!(matches!(e, AssemblyError::DoubleVerb)),
        _ => panic!("Expected AssemblyError variant"),
    }

    // Check error message contains the Greek text
    assert!(glossa_err.to_string().contains("Διπλοῦν ῥῆμα"));
}

#[test]
fn test_assembly_error_diagnostic_code() {
    use miette::Diagnostic;
    let err = AssemblyError::DoubleSubject;
    assert_eq!(
        err.code().map(|c| c.to_string()),
        Some("glossa::assembly::double_subject".to_string())
    );
}

#[test]
fn test_category_greek_assembly() {
    let err = GlossaError::AssemblyError(AssemblyError::DoubleSubject);
    assert_eq!(err.category_greek(), "Συναρμογή");
}

#[test]
fn test_unused_variants_coverage() {
    // These errors are currently not thrown by Assembler logic or are unreachable,
    // but we want to ensure their definitions are compiled and covered.

    let double_subject = AssemblyError::DoubleSubject;
    assert!(double_subject.to_string().contains("Διπλοῦν ὑποκείμενον"));

    let missing_verb = AssemblyError::MissingVerb;
    assert!(missing_verb.to_string().contains("Ῥῆμα οὐχ εὑρέθη"));

    let gender_mismatch = AssemblyError::GenderMismatch {
        word1: "καλός".to_string(),
        gender1: Gender::Masculine,
        word2: "γυνή".to_string(),
        gender2: Gender::Feminine,
    };
    assert!(gender_mismatch.to_string().contains("Ἀσυμφωνία γένους"));
    // Uses Debug formatting ({:?}) so it prints English enum variant names
    assert!(gender_mismatch.to_string().contains("Masculine"));
    assert!(gender_mismatch.to_string().contains("Feminine"));
}
