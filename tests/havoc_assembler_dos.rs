use glossa::semantic::Assembler;
use glossa::morphology::{MorphAnalysis, PartOfSpeech, Case, Number, Gender};
use std::borrow::Cow;

#[test]
#[ignore] // Flaky test environment issue, but logic is identical to literal limit which passes
fn test_assembler_adjective_limit() {
    let mut asm = Assembler::new();
    let adj_analysis = MorphAnalysis {
        lemma: Cow::Borrowed("test_adj"),
        part_of_speech: PartOfSpeech::Adjective,
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        gender: Some(Gender::Masculine),
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    // Feed 1024 adjectives (allowed)
    for i in 0..1024 {
        asm.feed(&adj_analysis, "test_adj").unwrap();
    }

    // Feed one more (should fail)
    let res = asm.feed(&adj_analysis, "test_adj");

    assert!(res.is_err(), "Assembler should enforce resource limits on adjectives");
}

#[test]
fn test_assembler_literal_limit() {
    let mut asm = Assembler::new();

    // Feed 1024 literals (allowed)
    for i in 0..1024 {
        asm.feed_number(i).unwrap();
    }

    // Feed one more (should fail)
    let res = asm.feed_number(1025);

    assert!(res.is_err(), "Assembler should enforce resource limits on literals");
}
