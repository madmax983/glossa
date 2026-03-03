use glossa::ast::{Expr, Word};
use glossa::morphology::{
    Case, Gender, MorphAnalysis, Number, PartOfSpeech, ParticipleAnalysis, Tense, Voice,
};
use glossa::semantic::Assembler;

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
        asm.feed(&adj_analysis, "καλός")
            .expect("Failed to feed adjective within limit");
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
        asm.feed_number(i)
            .expect("Failed to feed number within limit");
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
    asm.feed_string(safe_string)
        .expect("Failed to feed string within limit");

    // Create a string of length 65537 (should FAIL)
    let dangerous_string = "a".repeat(65537);
    let result = asm.feed_string(dangerous_string);
    assert!(
        result.is_err(),
        "Expected LimitExceeded error for string length > 65536, but got Ok"
    );
}

#[test]
fn test_assembler_array_limit() {
    let mut asm = Assembler::new();
    let empty_array = vec![];

    for _ in 0..32 {
        asm.feed_array(empty_array.clone()).unwrap();
    }

    let result = asm.feed_array(empty_array);
    assert!(result.is_err(), "Expected LimitExceeded for arrays");
}

#[test]
fn test_assembler_block_limit() {
    let mut asm = Assembler::new();
    let empty_block = vec![];

    for _ in 0..32 {
        asm.feed_block(empty_block.clone()).unwrap();
    }

    let result = asm.feed_block(empty_block);
    assert!(result.is_err(), "Expected LimitExceeded for blocks");
}

#[test]
fn test_assembler_nested_phrase_limit() {
    let mut asm = Assembler::new();
    let empty_phrase = vec![];

    for _ in 0..32 {
        asm.feed_nested_phrase(empty_phrase.clone()).unwrap();
    }

    let result = asm.feed_nested_phrase(empty_phrase);
    assert!(result.is_err(), "Expected LimitExceeded for nested phrases");
}

#[test]
fn test_assembler_index_access_limit() {
    let mut asm = Assembler::new();
    let array_expr = Expr::Word(Word::new("array"));
    let index_expr = Expr::NumberLiteral(0);

    for _ in 0..32 {
        asm.feed_index_access(array_expr.clone(), index_expr.clone())
            .unwrap();
    }

    let result = asm.feed_index_access(array_expr, index_expr);
    assert!(result.is_err(), "Expected LimitExceeded for index accesses");
}

#[test]
fn test_assembler_unwrap_limit() {
    let mut asm = Assembler::new();
    let expr = Expr::Word(Word::new("value"));

    for _ in 0..32 {
        asm.feed_unwrap(expr.clone()).unwrap();
    }

    let result = asm.feed_unwrap(expr);
    assert!(result.is_err(), "Expected LimitExceeded for unwraps");
}

#[test]
fn test_assembler_participle_limit() {
    let mut asm = Assembler::new();
    let participle_analysis = ParticipleAnalysis {
        stem: "γραφ".to_string(),
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Masculine,
        number: Number::Singular,
        confidence: 1.0,
    };

    for _ in 0..32 {
        asm.feed_participle(&participle_analysis, "γράφων").unwrap();
    }

    let result = asm.feed_participle(&participle_analysis, "γράφων");
    assert!(result.is_err(), "Expected LimitExceeded for participles");
}

#[test]
fn test_assembler_nominative_limit() {
    let mut asm = Assembler::new();
    let noun_analysis = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("ανθρωπος"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        gender: Some(Gender::Masculine),
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    // First one goes to `subject`
    asm.feed(&noun_analysis, "ἄνθρωπος").unwrap();

    // Next 32 go to `nominatives` (should be OK)
    for _ in 0..32 {
        asm.feed(&noun_analysis, "ἄνθρωπος").unwrap();
    }

    // 34th (33rd in `nominatives`) should FAIL
    let result = asm.feed(&noun_analysis, "ἄνθρωπος");
    assert!(result.is_err(), "Expected LimitExceeded for nominatives");
}

#[test]
fn test_assembler_genitive_limit() {
    let mut asm = Assembler::new();
    let noun_analysis = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("ανθρωπος"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Genitive),
        number: Some(Number::Singular),
        gender: Some(Gender::Masculine),
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    for _ in 0..32 {
        asm.feed(&noun_analysis, "ἀνθρώπου").unwrap();
    }

    let result = asm.feed(&noun_analysis, "ἀνθρώπου");
    assert!(result.is_err(), "Expected LimitExceeded for genitives");
}

#[test]
fn test_assembler_operator_limit() {
    let mut asm = Assembler::new();
    // Use "καί" (AND) operator, which triggers `check_operators`
    let and_particle = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("και"),
        part_of_speech: PartOfSpeech::Conjunction,
        case: None,
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    for _ in 0..32 {
        asm.feed(&and_particle, "καί").unwrap();
    }

    let result = asm.feed(&and_particle, "καί");
    assert!(result.is_err(), "Expected LimitExceeded for operators");
}

#[test]
fn test_assembler_property_access_limit_via_length() {
    let mut asm = Assembler::new();

    // Create subject first
    let subj = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("text"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        gender: Some(Gender::Neuter),
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    let length_prop = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("μηκος"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        gender: Some(Gender::Neuter),
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    for _ in 0..32 {
        asm.feed(&subj, "text").unwrap(); // Need subject for length property to apply
        asm.feed(&length_prop, "μῆκος").unwrap();
    }

    asm.feed(&subj, "text").unwrap();
    let result = asm.feed(&length_prop, "μῆκος");
    assert!(
        result.is_err(),
        "Expected LimitExceeded for property accesses"
    );
}
