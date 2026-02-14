use std::borrow::Cow;

use glossa::ast::{Expr, Word};
use glossa::morphology::{Case, Gender, MorphAnalysis, Number, PartOfSpeech};
use glossa::semantic::{Assembler, AssemblyError};

// Constants from src/semantic/assembler.rs
const MAX_ADJECTIVES: usize = 1024;
const MAX_LITERALS: usize = 1024;
const MAX_NOMINATIVES: usize = 256;
const MAX_GENITIVES: usize = 256;
const MAX_ARRAYS: usize = 256;
const MAX_INDEX_ACCESSES: usize = 256;
const MAX_PROPERTY_ACCESSES: usize = 256;
const MAX_NESTED_PHRASES: usize = 256;
const MAX_PARTICIPLES: usize = 256;
const MAX_UNWRAPS: usize = 256;
const MAX_BLOCKS: usize = 256;
const MAX_OPERATORS: usize = 256;

#[test]
fn test_limit_adjectives() {
    let mut asm = Assembler::new();
    let analysis = MorphAnalysis {
        lemma: Cow::Borrowed("adj"),
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

    for _ in 0..MAX_ADJECTIVES {
        asm.feed(&analysis, "adj").unwrap();
    }

    match asm.feed(&analysis, "adj") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Adjectives");
            assert_eq!(max, MAX_ADJECTIVES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_literals() {
    let mut asm = Assembler::new();
    for _ in 0..MAX_LITERALS {
        asm.feed_number(1).unwrap();
    }

    match asm.feed_number(1) {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Literals");
            assert_eq!(max, MAX_LITERALS);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_nominatives() {
    let mut asm = Assembler::new();
    let analysis = MorphAnalysis {
        lemma: Cow::Borrowed("noun"),
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

    // First nominative goes to subject slot
    asm.feed(&analysis, "noun").unwrap();

    // Subsequent nominatives go to nominatives list
    for _ in 0..MAX_NOMINATIVES {
        asm.feed(&analysis, "noun").unwrap();
    }

    match asm.feed(&analysis, "noun") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Nominatives");
            assert_eq!(max, MAX_NOMINATIVES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_genitives() {
    let mut asm = Assembler::new();
    let analysis = MorphAnalysis {
        lemma: Cow::Borrowed("gen"),
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

    for _ in 0..MAX_GENITIVES {
        asm.feed(&analysis, "gen").unwrap();
    }

    match asm.feed(&analysis, "gen") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Genitives");
            assert_eq!(max, MAX_GENITIVES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_arrays() {
    let mut asm = Assembler::new();
    for _ in 0..MAX_ARRAYS {
        asm.feed_array(vec![]).unwrap();
    }

    match asm.feed_array(vec![]) {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Arrays");
            assert_eq!(max, MAX_ARRAYS);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_index_accesses() {
    let mut asm = Assembler::new();
    let array = Expr::NumberLiteral(1);
    let index = Expr::NumberLiteral(0);

    for _ in 0..MAX_INDEX_ACCESSES {
        asm.feed_index_access(array.clone(), index.clone()).unwrap();
    }

    match asm.feed_index_access(array, index) {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Index Accesses");
            assert_eq!(max, MAX_INDEX_ACCESSES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_nested_phrases() {
    let mut asm = Assembler::new();
    for _ in 0..MAX_NESTED_PHRASES {
        asm.feed_nested_phrase(vec![]).unwrap();
    }

    match asm.feed_nested_phrase(vec![]) {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Nested Phrases");
            assert_eq!(max, MAX_NESTED_PHRASES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_unwraps() {
    let mut asm = Assembler::new();
    let expr = Expr::Word(Word::new("x"));

    for _ in 0..MAX_UNWRAPS {
        asm.feed_unwrap(expr.clone()).unwrap();
    }

    match asm.feed_unwrap(expr) {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Unwraps");
            assert_eq!(max, MAX_UNWRAPS);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_participles() {
    let mut asm = Assembler::new();
    let analysis = glossa::morphology::ParticipleAnalysis {
        stem: "stem".to_string(),
        tense: glossa::morphology::Tense::Present,
        voice: glossa::morphology::Voice::Active,
        case: Case::Nominative,
        gender: Gender::Masculine,
        number: Number::Singular,
        confidence: 1.0,
    };

    for _ in 0..MAX_PARTICIPLES {
        asm.feed_participle(&analysis, "part").unwrap();
    }

    match asm.feed_participle(&analysis, "part") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Participles");
            assert_eq!(max, MAX_PARTICIPLES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_blocks() {
    let mut asm = Assembler::new();
    for _ in 0..MAX_BLOCKS {
        asm.feed_block(vec![]).unwrap();
    }

    match asm.feed_block(vec![]) {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Blocks");
            assert_eq!(max, MAX_BLOCKS);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_operators() {
    // This tests the check_operators function which returns false on limit
    let mut asm = Assembler::new();
    // Use an analysis that triggers check_operators (e.g. "καί")
    let analysis = MorphAnalysis {
        lemma: Cow::Borrowed("και"),
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

    for _ in 0..MAX_OPERATORS {
        // "καί" is an operator
        asm.feed(&analysis, "καί").unwrap();
    }

    match asm.feed(&analysis, "καί") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Operators");
            assert_eq!(max, MAX_OPERATORS);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_property_accesses() {
    // This tests check_special_properties which returns false on limit
    let mut asm = Assembler::new();

    // Setup subject for property access
    let subj = MorphAnalysis {
        lemma: Cow::Borrowed("subj"),
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

    // We need to re-feed subject every time because property access consumes it!
    // But we can't because nominatives list is limited.
    // Actually, property access consumes `self.state.subject`.
    // So we need to feed subject, then property access, repeat.

    let prop_analysis = MorphAnalysis {
        lemma: Cow::Borrowed("μηκος"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Nominative), // irrelevant
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    for _ in 0..MAX_PROPERTY_ACCESSES {
        // Feed subject
        asm.feed(&subj, "text").unwrap();
        // Feed property "μῆκος" (length)
        asm.feed(&prop_analysis, "μῆκος").unwrap();
    }

    // Try one more
    asm.feed(&subj, "text").unwrap();
    match asm.feed(&prop_analysis, "μῆκος") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Property Accesses");
            assert_eq!(max, MAX_PROPERTY_ACCESSES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_string_method_properties() {
    // This tests try_create_string_method (called by check_method_verbs)
    let mut asm = Assembler::new();

    let subj = MorphAnalysis {
        lemma: Cow::Borrowed("subj"),
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

    // "κατά" preposition for delimiter
    let delimiter_prep = MorphAnalysis {
        lemma: Cow::Borrowed("κατα"),
        part_of_speech: PartOfSpeech::Preposition,
        case: None,
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    // "σχίζεται" (is split) verb
    let split_verb = MorphAnalysis {
        lemma: Cow::Borrowed("σχιζω"),
        part_of_speech: PartOfSpeech::Verb,
        case: None,
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    // Fill up property accesses first (using standard property access to fill buffer)
    let prop_analysis = MorphAnalysis {
        lemma: Cow::Borrowed("μηκος"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Nominative),
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    for _ in 0..MAX_PROPERTY_ACCESSES {
        asm.feed(&subj, "text").unwrap();
        asm.feed(&prop_analysis, "μῆκος").unwrap();
    }

    // Now try to trigger a string method which would add another property access
    asm.feed(&subj, "text").unwrap();
    asm.feed(&delimiter_prep, "κατά").unwrap();
    asm.feed_string(",".to_string()).unwrap(); // Delimiter literal

    // Feed split verb - should fail to create method call (return false)
    match asm.feed(&split_verb, "σχίζεται") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Property Accesses");
            assert_eq!(max, MAX_PROPERTY_ACCESSES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}
