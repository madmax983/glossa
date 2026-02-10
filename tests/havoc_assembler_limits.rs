use glossa::semantic::Assembler;
use glossa::morphology::{MorphAnalysis, PartOfSpeech, Case, Number, Gender};

/// 👺 Havoc: Resource Exhaustion in Assembler
///
/// This test verifies that the `Assembler` enforces strict resource limits
/// to prevent DoS attacks via unbounded vector growth.
///
/// If this test fails (panics), it means the limits are NOT enforced.
/// If it passes, it means the limits ARE enforced (or we are in the Red phase expecting failure).
#[test]
fn test_assembler_adjective_limit() {
    let mut asm = Assembler::new();
    let adj_analysis = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("καλος"),
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

    // Feed 32 adjectives (should be OK)
    for _ in 0..32 {
        asm.feed(&adj_analysis, "καλός").expect("Failed to feed adjective within limit");
    }

    // Feed 33rd adjective (should FAIL)
    let result = asm.feed(&adj_analysis, "καλός");
    assert!(
        result.is_err(),
        "Expected LimitExceeded error after 32 adjectives, but got Ok"
    );
}

#[test]
fn test_assembler_literal_limit() {
    let mut asm = Assembler::new();

    // Feed 32 numbers (should be OK)
    for i in 0..32 {
        asm.feed_number(i).expect("Failed to feed number within limit");
    }

    // Feed 33rd number (should FAIL)
    let result = asm.feed_number(33);
    assert!(
        result.is_err(),
        "Expected LimitExceeded error after 32 literals, but got Ok"
    );
}

#[test]
fn test_assembler_string_length_limit() {
    let mut asm = Assembler::new();

    // Create a string of length 65536 (should be OK)
    let safe_string = "a".repeat(65536);
    asm.feed_string(safe_string).expect("Failed to feed string within limit");

    // Create a string of length 65537 (should FAIL)
    let dangerous_string = "a".repeat(65537);
    let result = asm.feed_string(dangerous_string);
    assert!(
        result.is_err(),
        "Expected LimitExceeded error for string length > 65536, but got Ok"
    );
}
