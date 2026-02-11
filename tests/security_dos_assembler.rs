use glossa::ast::Expr;
use glossa::errors::assembly::AssemblyError;
use glossa::morphology::{Case, Gender, MorphAnalysis, Number, PartOfSpeech};
use glossa::semantic::Assembler;
use std::borrow::Cow;

// Limits (should match implementation)
const MAX_LITERALS: usize = 1024;
const MAX_ADJECTIVES: usize = 1024;
const MAX_STRING_LENGTH: usize = 65536;
const MAX_ARRAY_ELEMENTS: usize = 1024;
const MAX_NESTED_PHRASES: usize = 1024;
// Assume arrays count towards a limit, or have their own limit. Let's assume MAX_LITERALS applies to total literals?
// Or maybe just MAX_ARRAYS = 32.
const MAX_ARRAYS: usize = 1024;

#[test]
fn test_literal_limit() {
    let mut asm = Assembler::new();

    // Fill up to the limit
    for i in 0..MAX_LITERALS {
        asm.feed_number(i as i64).unwrap();
    }

    // Next one should fail
    let result = asm.feed_number(100);
    match result {
        Err(e) => {
            assert!(
                matches!(e, AssemblyError::LimitExceeded { ref limit_type, max } if limit_type == "literals" && max == MAX_LITERALS),
                "Expected LimitExceeded for literals, got {:?}",
                e
            );
            // Verify error message formatting for coverage
            assert!(e.to_string().contains("Ὑπέρβασις ὅρων"));
            assert!(e.to_string().contains("literals"));
        }
        Ok(_) => panic!("Expected LimitExceeded for literals, got Ok"),
    }
}

#[test]
fn test_string_length_limit() {
    let mut asm = Assembler::new();

    // Create a string that is too long
    let long_string = "a".repeat(MAX_STRING_LENGTH + 1);

    let result = asm.feed_string(long_string);
    assert!(
        matches!(result, Err(AssemblyError::LimitExceeded { ref limit_type, max })
            if limit_type == "string_length" && max == MAX_STRING_LENGTH),
        "Expected LimitExceeded for string length, got {:?}",
        result
    );
}

#[test]
fn test_adjective_limit() {
    let mut asm = Assembler::new();

    // Use a nonsense word to avoid triggering special lexicon rules (like numerals or operators)
    let adj = MorphAnalysis {
        lemma: Cow::Borrowed("agnostos"),
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

    // Fill up to the limit
    for _ in 0..MAX_ADJECTIVES {
        asm.feed(&adj, "ἄγνωστος").unwrap();
    }

    // Next one should fail
    let result = asm.feed(&adj, "ἄγνωστος");
    assert!(
        matches!(result, Err(AssemblyError::LimitExceeded { ref limit_type, max })
            if limit_type == "adjectives" && max == MAX_ADJECTIVES),
        "Expected LimitExceeded for adjectives, got {:?}",
        result
    );
}

#[test]
fn test_array_element_limit() {
    let mut asm = Assembler::new();

    // Feed one array with too many elements
    let huge_array: Vec<Expr> = (0..MAX_ARRAY_ELEMENTS + 1)
        .map(|_| Expr::NumberLiteral(1))
        .collect();
    let result = asm.feed_array(huge_array);
    assert!(
        matches!(result, Err(AssemblyError::LimitExceeded { ref limit_type, max })
            if limit_type == "array_elements" && max == MAX_ARRAY_ELEMENTS),
        "Expected LimitExceeded for array elements, got {:?}",
        result
    );
}

#[test]
fn test_array_count_limit() {
    let mut asm = Assembler::new();

    let array = vec![Expr::NumberLiteral(1)];
    for _ in 0..MAX_ARRAYS {
        asm.feed_array(array.clone()).unwrap();
    }

    let result = asm.feed_array(array);
    assert!(
        matches!(result, Err(AssemblyError::LimitExceeded { ref limit_type, max })
            if limit_type == "arrays" && max == MAX_ARRAYS),
        "Expected LimitExceeded for arrays count, got {:?}",
        result
    );
}

#[test]
fn test_nested_phrase_limit() {
    let mut asm = Assembler::new();

    // Fill nested phrases
    let terms = vec![Expr::NumberLiteral(1)];
    for _ in 0..MAX_NESTED_PHRASES {
        asm.feed_nested_phrase(terms.clone()).unwrap();
    }

    let result = asm.feed_nested_phrase(terms);
    assert!(
        matches!(result, Err(AssemblyError::LimitExceeded { ref limit_type, max })
            if limit_type == "nested_phrases" && max == MAX_NESTED_PHRASES),
        "Expected LimitExceeded for nested phrases, got {:?}",
        result
    );
}
