use glossa::morphology::{Case, Gender, MorphAnalysis, Number, PartOfSpeech, Person};
use glossa::semantic::{Assembler, AssemblyError};

#[test]
fn test_double_indirect_object_error() {
    let mut asm = Assembler::new();

    // First indirect object: τῷ ἀνθρώπῳ (to the man)
    let dat1 = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("ανθρωπος"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Dative),
        number: Some(Number::Singular),
        gender: Some(Gender::Masculine),
        person: Some(Person::Third),
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };
    asm.feed(&dat1, "ἀνθρώπῳ").unwrap();

    // Second indirect object: τῷ θεῷ (to the god)
    let dat2 = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("θεος"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Dative),
        number: Some(Number::Singular),
        gender: Some(Gender::Masculine),
        person: Some(Person::Third),
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    let result = asm.feed(&dat2, "θεῷ");

    // SENTRY: This assertion enforces that we DO NOT allow silent overwrites.
    // This test is expected to FAIL until the fix is applied.
    assert!(
        matches!(result, Err(AssemblyError::DoubleIndirect)),
        "Should return DoubleIndirect error, got {:?}",
        result
    );
}

#[test]
fn test_neuter_plural_agreement() {
    let mut asm = Assembler::new();

    // Subject: τὰ ζῷα (Neuter Plural)
    let subj = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("ζωον"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Nominative),
        number: Some(Number::Plural),
        gender: Some(Gender::Neuter),
        person: Some(Person::Third),
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };
    asm.feed(&subj, "ζῷα").unwrap();

    // Verb: τρέχουσιν (Plural)
    let verb = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("τρεχω"),
        part_of_speech: PartOfSpeech::Verb,
        case: None,
        number: Some(Number::Plural),
        gender: None,
        person: Some(Person::Third),
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };
    asm.feed(&verb, "τρέχουσιν").unwrap();

    let stmt = asm.finalize();
    assert!(
        stmt.is_ok(),
        "Neuter plural subject should allow plural verb (current behavior), got {:?}",
        stmt.err()
    );
}

#[test]
fn test_verbless_statement() {
    let mut asm = Assembler::new();

    // Subject: ὁ ἄνθρωπος
    let subj = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("ανθρωπος"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        gender: Some(Gender::Masculine),
        person: Some(Person::Third),
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };
    asm.feed(&subj, "ἄνθρωπος").unwrap();

    // No verb fed.

    let stmt = asm.finalize();
    // Current behavior allows verbless statements if they have content.
    assert!(
        stmt.is_ok(),
        "Verbless statement with content should be allowed"
    );
    let s = stmt.unwrap();
    assert!(s.subject.is_some());
    assert!(s.verb.is_none());
}
