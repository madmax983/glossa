
use std::borrow::Cow;
use glossa::morphology::{MorphAnalysis, PartOfSpeech, Case, Number, Gender};
use glossa::semantic::{Assembler, AssemblyError};
use glossa::semantic::assembler::{MAX_OPERATORS, MAX_INDEX_ACCESSES};

#[test]
fn test_limit_operators_or() {
    // This tests the OR path in check_operators
    let mut asm = Assembler::new();
    let analysis = MorphAnalysis {
        lemma: Cow::Borrowed("η"),
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
        asm.feed(&analysis, "ἤ").unwrap();
    }

    match asm.feed(&analysis, "ἤ") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Operators");
            assert_eq!(max, MAX_OPERATORS);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_operators_comparison() {
    // This tests the Comparison path in check_operators
    let mut asm = Assembler::new();
    let analysis = MorphAnalysis {
        lemma: Cow::Borrowed("μειζον"),
        part_of_speech: PartOfSpeech::Adjective,
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
        asm.feed(&analysis, "μεῖζον").unwrap();
    }

    match asm.feed(&analysis, "μεῖζον") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Operators");
            assert_eq!(max, MAX_OPERATORS);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_operators_arithmetic() {
    // This tests the Arithmetic path in check_operators
    let mut asm = Assembler::new();
    let analysis = MorphAnalysis {
        lemma: Cow::Borrowed("αθροισμα"),
        part_of_speech: PartOfSpeech::Noun,
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
        asm.feed(&analysis, "ἄθροισμα").unwrap();
    }

    match asm.feed(&analysis, "ἄθροισμα") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Operators");
            assert_eq!(max, MAX_OPERATORS);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_ordinal_index() {
    // This tests the ordinal adjective path in check_special_properties
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

    let ordinal = MorphAnalysis {
        lemma: Cow::Borrowed("πρωτον"),
        part_of_speech: PartOfSpeech::Adjective,
        case: None,
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    for _ in 0..MAX_INDEX_ACCESSES {
        asm.feed(&subj, "text").unwrap();
        asm.feed(&ordinal, "πρῶτον").unwrap();
    }

    asm.feed(&subj, "text").unwrap();
    match asm.feed(&ordinal, "πρῶτον") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Index Accesses");
            assert_eq!(max, MAX_INDEX_ACCESSES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}
