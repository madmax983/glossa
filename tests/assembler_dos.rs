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
fn test_assembler_genitive_limit() {
    let mut asm = Assembler::new();

    let gen_analysis = MorphAnalysis {
        lemma: Cow::Borrowed("ονομα"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Genitive),
        number: Some(Number::Singular),
        gender: Some(Gender::Neuter),
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    // Feed 16 genitives (limit is 16)
    for _ in 0..16 {
        asm.feed(&gen_analysis, "ὀνόματος").unwrap();
    }

    // Try to feed one more
    let result = asm.feed(&gen_analysis, "ὀνόματος");

    assert!(result.is_err(), "Should enforce limit on genitives");
    assert!(matches!(
        result.unwrap_err(),
        AssemblyError::LimitExceeded { .. }
    ));
}

#[test]
fn test_assembler_array_limit() {
    use glossa::ast::Expr;
    let mut asm = Assembler::new();

    // Feed 16 arrays (limit is 16)
    for _ in 0..16 {
        asm.feed_array(vec![Expr::NumberLiteral(1)]).unwrap();
    }

    // Try to feed one more
    let result = asm.feed_array(vec![Expr::NumberLiteral(1)]);

    assert!(result.is_err(), "Should enforce limit on arrays");
    assert!(matches!(
        result.unwrap_err(),
        AssemblyError::LimitExceeded { .. }
    ));
}

#[test]
fn test_assembler_index_access_limit() {
    use glossa::ast::{Expr, Word};
    let mut asm = Assembler::new();

    let array = Expr::Word(Word {
        original: "A".into(),
        normalized: "A".into(),
    });
    let index = Expr::NumberLiteral(0);

    // Feed 16 index accesses (limit is 16)
    for _ in 0..16 {
        asm.feed_index_access(array.clone(), index.clone()).unwrap();
    }

    // Try to feed one more
    let result = asm.feed_index_access(array, index);

    assert!(result.is_err(), "Should enforce limit on index accesses");
    assert!(matches!(
        result.unwrap_err(),
        AssemblyError::LimitExceeded { .. }
    ));
}

#[test]
fn test_assembler_block_limit() {
    let mut asm = Assembler::new();

    // Feed 16 blocks (limit is 16)
    for _ in 0..16 {
        asm.feed_block(vec![]).unwrap();
    }

    // Try to feed one more
    let result = asm.feed_block(vec![]);

    assert!(result.is_err(), "Should enforce limit on blocks");
    assert!(matches!(
        result.unwrap_err(),
        AssemblyError::LimitExceeded { .. }
    ));
}

#[test]
fn test_assembler_nested_phrase_limit() {
    use glossa::ast::Expr;
    let mut asm = Assembler::new();

    // Feed 16 nested phrases (limit is 16)
    for _ in 0..16 {
        asm.feed_nested_phrase(vec![Expr::NumberLiteral(1)])
            .unwrap();
    }

    // Try to feed one more
    let result = asm.feed_nested_phrase(vec![Expr::NumberLiteral(1)]);

    assert!(result.is_err(), "Should enforce limit on nested phrases");
    assert!(matches!(
        result.unwrap_err(),
        AssemblyError::LimitExceeded { .. }
    ));
}

#[test]
fn test_assembler_unwrap_limit() {
    use glossa::ast::{Expr, Word};
    let mut asm = Assembler::new();

    let expr = Expr::Word(Word {
        original: "A".into(),
        normalized: "A".into(),
    });

    // Feed 16 unwraps (limit is 16)
    for _ in 0..16 {
        asm.feed_unwrap(expr.clone()).unwrap();
    }

    // Try to feed one more
    let result = asm.feed_unwrap(expr);

    assert!(result.is_err(), "Should enforce limit on unwraps");
    assert!(matches!(
        result.unwrap_err(),
        AssemblyError::LimitExceeded { .. }
    ));
}

#[test]
fn test_assembler_participle_limit() {
    use glossa::morphology::{ParticipleAnalysis, Tense, Voice};
    let mut asm = Assembler::new();

    let analysis = ParticipleAnalysis {
        stem: "stem".into(),
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Masculine,
        number: Number::Singular,
        confidence: 1.0,
    };

    // Feed 16 participles (limit is 16)
    for _ in 0..16 {
        asm.feed_participle(&analysis, "word").unwrap();
    }

    // Try to feed one more
    let result = asm.feed_participle(&analysis, "word");

    assert!(result.is_err(), "Should enforce limit on participles");
    assert!(matches!(
        result.unwrap_err(),
        AssemblyError::LimitExceeded { .. }
    ));
}

#[test]
fn test_assembler_operator_limit() {
    let mut asm = Assembler::new();

    let analysis = MorphAnalysis {
        lemma: Cow::Borrowed("kai"),
        part_of_speech: PartOfSpeech::Conjunction, // Assuming operator check handles this
        case: None,
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    // Feed 32 operators (limit is 32)
    for _ in 0..32 {
        asm.feed(&analysis, "καί").unwrap();
    }

    // Try to feed one more
    let result = asm.feed(&analysis, "καί");

    assert!(result.is_err(), "Should enforce limit on operators");
    assert!(matches!(
        result.unwrap_err(),
        AssemblyError::LimitExceeded { .. }
    ));
}

#[test]
fn test_assembler_nominative_limit() {
    let mut asm = Assembler::new();

    let subj_analysis = MorphAnalysis {
        lemma: Cow::Borrowed("anthropos"),
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

    // Feed 1 subject (fills state.subject)
    asm.feed(&subj_analysis, "ἄνθρωπος").unwrap();

    // Feed 16 more nominatives (fills state.nominatives, limit is 16)
    for _ in 0..16 {
        asm.feed(&subj_analysis, "ἄνθρωπος").unwrap();
    }

    // Try to feed one more
    let result = asm.feed(&subj_analysis, "ἄνθρωπος");

    assert!(result.is_err(), "Should enforce limit on nominatives");
    assert!(matches!(
        result.unwrap_err(),
        AssemblyError::LimitExceeded { .. }
    ));
}

#[test]
fn test_assembler_property_access_limit() {
    let mut asm = Assembler::new();

    // We need a subject for property access to work
    let subj_analysis = MorphAnalysis {
        lemma: Cow::Borrowed("logos"),
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

    // Property noun "μῆκος" (length)
    let len_analysis = MorphAnalysis {
        lemma: Cow::Borrowed("mekos"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Nominative), // Case doesn't matter for property check
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    // Feed 16 property accesses (limit is 16)
    for _ in 0..16 {
        // Must provide a subject first
        asm.feed(&subj_analysis, "λόγος").unwrap();
        // Then the property noun triggers access
        asm.feed(&len_analysis, "μῆκος").unwrap();
    }

    // Try to feed one more
    asm.feed(&subj_analysis, "λόγος").unwrap();
    let result = asm.feed(&len_analysis, "μῆκος");

    assert!(result.is_err(), "Should enforce limit on property accesses");
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
