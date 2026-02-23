use super::AssemblyError;
use super::assembler::Assembler;
use crate::morphology::{
    Case, Gender, Mood, MorphAnalysis, Number, PartOfSpeech, Person, Tense, Voice,
};
use std::borrow::Cow;

#[test]
fn test_huge_string_literal_rejection() {
    let mut asm = Assembler::new();
    // 65537 bytes (1 byte over 65536 limit)
    let huge_string = "a".repeat(65537);

    let result = asm.feed_string(huge_string);

    // We expect this to FAIL until implemented
    assert!(
        matches!(result, Err(AssemblyError::LimitExceeded { ref resource, .. }) if resource == "String Literal Length")
    );
}

#[test]
fn test_huge_identifier_rejection() {
    let mut asm = Assembler::new();
    // 257 bytes (1 byte over 256 limit)
    let huge_ident = "a".repeat(257);
    let analysis = MorphAnalysis {
        lemma: Cow::Borrowed("lemma"),
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

    let result = asm.feed(&analysis, &huge_ident);

    // We expect this to FAIL until implemented
    assert!(
        matches!(result, Err(AssemblyError::LimitExceeded { ref resource, .. }) if resource == "Identifier Length")
    );
}

#[test]
fn test_neuter_plural_subject_plural_verb_rejection() {
    let mut asm = Assembler::new();

    // Subject: Neuter Plural (e.g. "dwra" - gifts)
    // We use "δῶρα"
    let subj = MorphAnalysis {
        lemma: Cow::Borrowed("dwron"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Nominative),
        number: Some(Number::Plural), // PLURAL
        gender: Some(Gender::Neuter), // NEUTER
        person: Some(Person::Third),
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };
    asm.feed(&subj, "δῶρα").unwrap();

    // Verb: Plural (e.g. "blepousin" - they see)
    // We use "βλέπουσιν"
    let verb = MorphAnalysis {
        lemma: Cow::Borrowed("blepw"),
        part_of_speech: PartOfSpeech::Verb,
        case: None,
        number: Some(Number::Plural), // PLURAL
        gender: None,
        person: Some(Person::Third),
        tense: Some(Tense::Present),
        mood: Some(Mood::Indicative),
        voice: Some(Voice::Active),
        confidence: 1.0,
    };

    // This SHOULD fail strictly, as Neuter Plural takes Singular Verb
    let result = asm.feed(&verb, "βλέπουσιν");

    // Currently this PASSES (returns Ok) because the check is loose.
    // We want it to FAIL (return Err).
    assert!(
        matches!(result, Err(AssemblyError::SubjectVerbDisagreement { .. })),
        "Expected rejection of Plural Verb for Neuter Plural Subject, but got {:?}",
        result
    );
}
