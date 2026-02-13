use glossa::semantic::{Assembler, AssemblyError};
use glossa::morphology::{Case, Gender, MorphAnalysis, Number, PartOfSpeech};
use std::borrow::Cow;

#[test]
fn test_assembler_enforces_limits() {
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

    // Fill up to the limit (1024)
    for _ in 0..1024 {
        let res = asm.feed(&adj_analysis, "test_adj");
        assert!(res.is_ok(), "Should accept up to 1024 adjectives");
    }

    // Attempt to exceed the limit
    let res = asm.feed(&adj_analysis, "test_adj");

    // Sentry: Assert that we get the correct LimitExceeded error
    match res {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Adjectives");
            assert_eq!(max, 1024);
        }
        Err(e) => panic!("Expected LimitExceeded, got {:?}", e),
        Ok(_) => panic!("Assembler allowed unbounded growth! Expected LimitExceeded error."),
    }
}
