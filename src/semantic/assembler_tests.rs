use crate::ast::{Expr, Word};
use crate::morphology::{Case, Gender, MorphAnalysis, Number, PartOfSpeech, Person, analyze};
use crate::semantic::AssemblyError;
use crate::semantic::assembler::{
    Assembler, MAX_ADJECTIVES, MAX_ARRAYS, MAX_BLOCKS, MAX_GENITIVES, MAX_INDEX_ACCESSES,
    MAX_LITERALS, MAX_NESTED_PHRASES, MAX_NOMINATIVES, MAX_OPERATORS, MAX_PARTICIPLES,
    MAX_PROPERTY_ACCESSES, MAX_UNWRAPS,
};
use std::borrow::Cow;

// From semantic_assembler_edge_cases.rs

#[test]
fn test_split_verb_consumes_literal_without_subject() {
    let mut asm = Assembler::new();

    // 1. Feed "κατά" (delimiter preposition)
    let kata = analyze("κατα"); // Should be preposition
    asm.feed(&kata, "κατά").unwrap();

    // 2. Feed string literal " "
    asm.feed_string(" ".to_string()).unwrap();

    // 3. Feed "σχίζεται" (split verb)
    // This triggers check_method_verbs
    let split = analyze("σχιζεται");
    asm.feed(&split, "σχίζεται").unwrap();

    // At this point, if the bug exists:
    // - check_method_verbs returned true (so it was "handled")
    // - pending_literals.pop() was called and consumed " "
    // - pending_subject was None, so no property access was created.

    // 4. Feed "λόγος" (subject)
    let subject = analyze("λογος");
    asm.feed(&subject, "λόγος").unwrap();

    // 5. Feed "λέγε" (print verb)
    // REMOVED: Since "split" is now correctly identified as a verb when the pattern match fails,
    // adding another verb would cause a DoubleVerb error.
    // let verb = analyze("λεγε");
    // asm.feed(&verb, "λέγε").unwrap();

    let stmt = asm.finalize().unwrap();

    // If the literal was consumed by the failed split pattern match, it will be missing.
    // If it was preserved, it should be in stmt.literals.
    assert!(
        !stmt.literals.is_empty(),
        "Literal should not be consumed if split pattern fails to match due to missing subject"
    );

    // Also verify that "split" was captured as the verb
    assert!(stmt.verb.is_some(), "Split should be captured as the verb");
    assert_eq!(stmt.verb.unwrap().lemma, "σχιζω");
}

#[test]
fn test_split_verb_not_ignored_without_delimiter() {
    let mut asm = Assembler::new();

    // Feed subject "word"
    let subj = analyze("λογος"); // "word"
    asm.feed(&subj, "λόγος").unwrap();

    // Feed "splits" (σχίζει) without "by" (κατά) and delimiter string
    // normalized: σχιζει
    // This should now be treated as a normal verb because the split pattern didn't match!
    let split_verb = analyze("σχιζει");
    asm.feed(&split_verb, "σχίζει").unwrap();

    let stmt = asm.finalize();

    match stmt {
        Ok(s) => {
            // Now we expect the verb to be present!
            assert!(
                s.verb.is_some(),
                "Verb should be present (treated as normal verb) when split pattern fails"
            );
            let verb = s.verb.unwrap();
            assert_eq!(verb.original, "σχίζει");
            assert!(s.string_method.is_none(), "String method should be None");
        }
        Err(e) => {
            panic!("Should not error: {:?}", e);
        }
    }
}

#[test]
fn test_ordinal_not_ignored_without_subject() {
    let mut asm = Assembler::new();

    // Feed "first" (πρῶτον) - Ordinal
    // normalized: πρωτον
    // Since there is no subject yet, it should fall through and be treated as an Adjective
    let first = analyze("πρωτον");
    asm.feed(&first, "πρῶτον").unwrap();

    // Feed "man" (ἄνθρωπος) - Subject
    let man = analyze("ανθρωπος");
    asm.feed(&man, "ἄνθρωπος").unwrap();

    // Feed "is" (ἐστί) - Verb
    let is_verb = analyze("εστι");
    asm.feed(&is_verb, "ἐστί").unwrap();

    let stmt = asm.finalize().unwrap();

    assert!(stmt.subject.is_some(), "Subject should be present");
    assert_eq!(stmt.subject.unwrap().original, "ἄνθρωπος");

    // "first" should be in adjectives now!
    assert!(
        !stmt.adjectives.is_empty(),
        "Adjectives should NOT be empty; 'first' should be captured"
    );
    assert_eq!(stmt.adjectives[0].original, "πρῶτον");

    assert!(
        stmt.index_accesses.is_empty(),
        "Index accesses should be empty"
    );
}

#[test]
fn test_length_property_not_ignored_without_subject() {
    let mut asm = Assembler::new();

    // Feed "length" (μῆκος) - Noun
    // normalized: μηκος
    // Since there is no subject, it should fall through and be treated as a Noun (Subject/Object)
    let len = analyze("μηκος");
    asm.feed(&len, "μῆκος").unwrap();

    // Feed "is" (ἐστί)
    let is_verb = analyze("εστι");
    asm.feed(&is_verb, "ἐστί").unwrap();

    // Feed "5"
    asm.feed_number(5).unwrap();

    let stmt = asm.finalize().unwrap();

    // "length" should be the subject now!
    assert!(stmt.subject.is_some(), "Subject should be present (length)");
    assert_eq!(stmt.subject.unwrap().lemma, "μηκος");

    assert!(
        stmt.property_accesses.is_empty(),
        "Property accesses should be empty"
    );
}

// From havoc_assembler_dos.rs

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
    let analysis = crate::morphology::ParticipleAnalysis {
        stem: "stem".to_string(),
        tense: crate::morphology::Tense::Present,
        voice: crate::morphology::Voice::Active,
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

// From havoc_assembler_dos_extra.rs

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

// From sentry_assembler_tests.rs

#[test]
fn test_double_indirect_object_error() {
    let mut asm = Assembler::new();

    // First indirect object: τῷ ἀνθρώπῳ (to the man)
    let dat1 = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("ανθρωπος"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Dative),
        number: Some(Number::Singular),
        gender: Some(Gender::Masculine),
        person: Some(Person::Third),
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };
    asm.feed(&dat1, "ἀνθρώπῳ").unwrap();

    // Second indirect object: τῷ θεῷ (to the god)
    let dat2 = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("θεος"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Dative),
        number: Some(Number::Singular),
        gender: Some(Gender::Masculine),
        person: Some(Person::Third),
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    let result = asm.feed(&dat2, "θεῷ");

    // SENTRY: This assertion enforces that we DO NOT allow silent overwrites.
    assert!(
        matches!(result, Err(AssemblyError::DoubleIndirect)),
        "Should return DoubleIndirect error, got {:?}",
        result
    );
}

#[test]
fn test_neuter_plural_agreement() {
    let mut asm = Assembler::new();

    // Subject: τὰ ζῷα (Neuter Plural)
    let subj = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("ζωον"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Nominative),
        number: Some(Number::Plural),
        gender: Some(Gender::Neuter),
        person: Some(Person::Third),
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };
    asm.feed(&subj, "ζῷα").unwrap();

    // Verb: τρέχουσιν (Plural)
    let verb = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("τρεχω"),
        part_of_speech: PartOfSpeech::Verb,
        case: None,
        number: Some(Number::Plural),
        gender: None,
        person: Some(Person::Third),
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };
    asm.feed(&verb, "τρέχουσιν").unwrap();

    let stmt = asm.finalize();
    assert!(
        stmt.is_ok(),
        "Neuter plural subject should allow plural verb (current behavior), got {:?}",
        stmt.err()
    );
}

#[test]
fn test_verbless_statement() {
    let mut asm = Assembler::new();

    // Subject: ὁ ἄνθρωπος
    let subj = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("ανθρωπος"),
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
    asm.feed(&subj, "ἄνθρωπος").unwrap();

    // No verb fed.

    let stmt = asm.finalize();
    // Current behavior allows verbless statements if they have content.
    assert!(
        stmt.is_ok(),
        "Verbless statement with content should be allowed"
    );
    let s = stmt.unwrap();
    assert!(s.subject.is_some());
    assert!(s.verb.is_none());
}

// From atlas_refactor_coverage.rs

#[test]
fn test_assembled_statement_derive_coverage() {
    use crate::semantic::assembly_model::AssembledStatement;
    // Cover #[derive(Clone, Debug, Default)] for AssembledStatement
    let stmt = AssembledStatement::default();
    let cloned = stmt.clone();
    let debug_str = format!("{:?}", cloned);
    assert!(debug_str.contains("AssembledStatement"));

    // Cover internal fields being None/Empty by default
    assert!(stmt.subject.is_none());
    assert!(stmt.nominatives.is_empty());
}

#[test]
fn test_assembler_special_markers_coverage() {
    let mut asm = Assembler::new();

    // "μετά" (mutable marker)
    let meta = analyze("μετα"); // Preposition
    asm.feed(&meta, "μετά").unwrap();
    let stmt = asm.finalize().unwrap();
    assert!(stmt.has_mutable_marker);

    // "ἐν" (containment)
    let en = analyze("εν"); // Preposition
    asm.feed(&en, "ἐν").unwrap();
    let stmt2 = asm.finalize().unwrap();
    assert!(stmt2.has_containment_preposition);

    // "κατά" (delimiter)
    let kata = analyze("κατα"); // Preposition
    asm.feed(&kata, "κατά").unwrap();
    let stmt3 = asm.finalize().unwrap();
    assert!(stmt3.has_delimiter_preposition);
}

#[test]
fn test_assembler_method_verbs_join_coverage() {
    let mut asm = Assembler::new();

    // 1. Subject: "list"
    let list = analyze("λιστη");
    asm.feed(&list, "λίστη").unwrap();

    // 2. Delimiter Preposition: "κατά"
    let kata = analyze("κατα");
    asm.feed(&kata, "κατά").unwrap();

    // 3. Delimiter Literal: ","
    asm.feed_string(",".to_string()).unwrap();

    // 4. Join Verb: "ἑνοῦνται"
    let join = analyze("ενουνται");
    asm.feed(&join, "ἑνοῦνται").unwrap();

    let stmt = asm.finalize().unwrap();
    assert_eq!(
        stmt.string_method,
        Some(("join".to_string(), ",".to_string()))
    );
}

#[test]
fn test_assembler_arithmetic_operators_coverage() {
    let mut asm = Assembler::new();

    // "ἄθροισμα" (sum -> +)
    let sum = analyze("αθροισμα");
    asm.feed(&sum, "ἄθροισμα").unwrap();

    let stmt = asm.finalize().unwrap();
    assert!(!stmt.operators.is_empty());
}

#[test]
fn test_assembler_boolean_and_coverage() {
    let mut asm = Assembler::new();

    // "καί" (and)
    let and = analyze("και");
    asm.feed(&and, "καί").unwrap();

    let stmt = asm.finalize().unwrap();
    assert!(!stmt.operators.is_empty());
}

#[test]
fn test_assembler_numeral_coverage() {
    let mut asm = Assembler::new();

    // "πέντε" (5)
    let five = analyze("πεντε");
    asm.feed(&five, "πέντε").unwrap();

    let stmt = asm.finalize().unwrap();
    assert_eq!(stmt.literals.len(), 1);
}

#[test]
fn test_assembler_set_flags_coverage() {
    let mut asm = Assembler::new();
    asm.set_query(true);
    asm.set_propagate(true);

    let stmt = asm.finalize().unwrap();
    assert!(stmt.is_query);
    assert!(stmt.is_propagate);
}

// test_assembler_has_content_coverage removed as has_content was removed

#[test]
fn test_assembler_method_verbs_split_coverage() {
    let mut asm = Assembler::new();

    // 1. Subject: "string"
    let string_noun = analyze("λογος");
    asm.feed(&string_noun, "λόγος").unwrap();

    // 2. Delimiter Preposition: "κατά"
    let kata = analyze("κατα");
    asm.feed(&kata, "κατά").unwrap();

    // 3. Delimiter Literal: "."
    asm.feed_string(".".to_string()).unwrap();

    // 4. Split Verb: "σχίζεται"
    let split = analyze("σχιζεται");
    asm.feed(&split, "σχίζεται").unwrap();

    let stmt = asm.finalize().unwrap();
    assert_eq!(
        stmt.string_method,
        Some(("split".to_string(), ".".to_string()))
    );
}

#[test]
fn test_assembler_ordinal_index_coverage() {
    let mut asm = Assembler::new();

    // 1. Subject: "array" (use known noun "λιστη")
    let array = analyze("λιστη");
    asm.feed(&array, "λίστη").unwrap();

    // 2. Ordinal: "first" (should map to index 0)
    let first = analyze("πρωτον");
    asm.feed(&first, "πρῶτον").unwrap();

    let stmt = asm.finalize().unwrap();
    assert_eq!(stmt.index_accesses.len(), 1);
    // Subject should be consumed
    assert!(stmt.subject.is_none());
}

#[test]
fn test_assembler_error_cases_coverage() {
    let mut asm = Assembler::new();

    // Double Verb
    let verb1 = analyze("λεγει");
    asm.feed(&verb1, "λέγει").unwrap();
    let result = asm.feed(&verb1, "λέγει");
    assert!(matches!(result, Err(AssemblyError::DoubleVerb)));
    let _ = asm.finalize();

    // Double Object
    let obj = analyze("λογον");
    asm.feed(&obj, "λόγον").unwrap();
    let result = asm.feed(&obj, "λόγον");
    assert!(matches!(result, Err(AssemblyError::DoubleObject)));
    let _ = asm.finalize();

    // Double Indirect
    let ind = analyze("ανθρωπω");
    asm.feed(&ind, "ἀνθρώπῳ").unwrap();
    let result = asm.feed(&ind, "ἀνθρώπῳ");
    assert!(matches!(result, Err(AssemblyError::DoubleIndirect)));
}

#[test]
fn test_constituent_derive_coverage() {
    use crate::morphology::Case;
    use crate::semantic::assembly_model::Constituent;
    use smol_str::SmolStr;

    let c = Constituent {
        lemma: SmolStr::new("test"),
        original: SmolStr::new("test"),
        case: Case::Nominative,
        number: None,
        gender: None,
        person: None,
    };

    let cloned = c.clone();
    let debug = format!("{:?}", cloned);
    assert!(debug.contains("test"));
}
