use glossa::morphology::{Case, Gender, MorphAnalysis, Number, PartOfSpeech};
use glossa::semantic::{Assembler, AssemblyError};
use std::borrow::Cow;

#[test]
fn test_assembler_adjective_limit() {
    let mut asm = Assembler::new();

    // Create a dummy adjective analysis
    let adj_analysis = MorphAnalysis {
        lemma: Cow::Borrowed("καλος"),
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

    // Feed a reasonable number of adjectives (e.g., 20) - should pass
    // Limit is 32, so 20 is fine
    for _ in 0..20 {
        asm.feed(&adj_analysis, "καλός").unwrap();
    }

    // Now try to feed beyond limit
    // We already have 20. Try to add 20 more.
    let result = (0..20).try_for_each(|_| asm.feed(&adj_analysis, "καλός"));

    // Should fail with LimitExceeded
    assert!(result.is_err(), "Should enforce limit on adjectives");
    assert!(matches!(
        result.unwrap_err(),
        AssemblyError::LimitExceeded { .. }
    ));
}

#[test]
fn test_assembler_literal_limit() {
    let mut asm = Assembler::new();

    // Limit is 32.
    // Feed 40 string literals
    let result = (0..40).try_for_each(|_| asm.feed_string("test".to_string()));

    assert!(result.is_err(), "Should enforce limit on literals");
    assert!(matches!(
        result.unwrap_err(),
        AssemblyError::LimitExceeded { .. }
    ));
}

#[test]
fn test_assembler_string_length_limit() {
    let mut asm = Assembler::new();

    // Create a huge string (64KB + 1)
    let huge_string = "a".repeat(65537);

    let result = asm.feed_string(huge_string);

    assert!(result.is_err(), "Should enforce limit on string length");
    assert!(matches!(
        result.unwrap_err(),
        AssemblyError::LimitExceeded { .. }
    ));
}
