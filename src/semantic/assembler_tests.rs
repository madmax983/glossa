#![allow(unused_mut)]
use crate::ast::{Expr, Word};
use crate::morphology::{Case, Gender, MorphAnalysis, Number, PartOfSpeech, Person, analyze};
use crate::semantic::AssemblyError;
use crate::semantic::assembly::{
    AssembledStatement, MAX_ADJECTIVES, MAX_ARRAYS, MAX_BLOCKS, MAX_GENITIVES, MAX_INDEX_ACCESSES,
    MAX_LITERALS, MAX_NESTED_PHRASES, MAX_NOMINATIVES, MAX_OPERATORS, MAX_PARTICIPLES,
    MAX_PROPERTY_ACCESSES, MAX_UNWRAPS,
};
use std::borrow::Cow;

// From semantic_assembler_edge_cases.rs

#[test]
fn test_split_verb_consumes_literal_without_subject() {
    let mut stmt = AssembledStatement::new();

    // 1. Feed "κατά" (delimiter preposition)
    let kata = analyze("κατα"); // Should be preposition
    stmt.feed(&kata, "κατά").unwrap();

    // 2. Feed string literal " "
    stmt.feed_string(" ".to_string()).unwrap();

    // 3. Feed "σχίζεται" (split verb)
    // This triggers check_method_verbs
    let split = analyze("σχιζεται");
    stmt.feed(&split, "σχίζεται").unwrap();

    // At this point, if the bug exists:
    // - check_method_verbs returned true (so it was "handled")
    // - pending_literals.pop() was called and consumed " "
    // - pending_subject was None, so no property access was created.

    // 4. Feed "λόγος" (subject)
    let subject = analyze("λογος");
    stmt.feed(&subject, "λόγος").unwrap();

    // 5. Feed "λέγε" (print verb)
    // REMOVED: Since "split" is now correctly identified as a verb when the pattern match fails,
    // adding another verb would cause a DoubleVerb error.
    // let verb = analyze("λεγε");
    // stmt.feed(&verb, "λέγε").unwrap();

    let mut stmt = stmt.clone();

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
    let mut stmt = AssembledStatement::new();

    // Feed subject "word"
    let subj = analyze("λογος"); // "word"
    stmt.feed(&subj, "λόγος").unwrap();

    // Feed "splits" (σχίζει) without "by" (κατά) and delimiter string
    // normalized: σχιζει
    // This should now be treated as a normal verb because the split pattern didn't match!
    let split_verb = analyze("σχιζει");
    stmt.feed(&split_verb, "σχίζει").unwrap();

    let stmt = stmt.finalize();

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
    let mut stmt = AssembledStatement::new();

    // Feed "first" (πρῶτον) - Ordinal
    // normalized: πρωτον
    // Since there is no subject yet, it should fall through and be treated as an Adjective
    let first = analyze("πρωτον");
    stmt.feed(&first, "πρῶτον").unwrap();

    // Feed "man" (ἄνθρωπος) - Subject
    let man = analyze("ανθρωπος");
    stmt.feed(&man, "ἄνθρωπος").unwrap();

    // Feed "is" (ἐστί) - Verb
    let is_verb = analyze("εστι");
    stmt.feed(&is_verb, "ἐστί").unwrap();

    let mut stmt = stmt.clone();

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
    let mut stmt = AssembledStatement::new();

    // Feed "length" (μῆκος) - Noun
    // normalized: μηκος
    // Since there is no subject, it should fall through and be treated as a Noun (Subject/Object)
    let len = analyze("μηκος");
    stmt.feed(&len, "μῆκος").unwrap();

    // Feed "is" (ἐστί)
    let is_verb = analyze("εστι");
    stmt.feed(&is_verb, "ἐστί").unwrap();

    // Feed "5"
    stmt.feed_number(5).unwrap();

    let mut stmt = stmt.clone();

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
    let mut stmt = AssembledStatement::new();
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
        stmt.feed(&analysis, "adj").unwrap();
    }

    match stmt.feed(&analysis, "adj") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Adjectives");
            assert_eq!(max, MAX_ADJECTIVES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_literals() {
    let mut stmt = AssembledStatement::new();
    for _ in 0..MAX_LITERALS {
        stmt.feed_number(1).unwrap();
    }

    match stmt.feed_number(1) {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Literals");
            assert_eq!(max, MAX_LITERALS);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_nominatives() {
    let mut stmt = AssembledStatement::new();
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
    stmt.feed(&analysis, "noun").unwrap();

    // Subsequent nominatives go to nominatives list
    for _ in 0..MAX_NOMINATIVES {
        stmt.feed(&analysis, "noun").unwrap();
    }

    match stmt.feed(&analysis, "noun") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Nominatives");
            assert_eq!(max, MAX_NOMINATIVES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_genitives() {
    let mut stmt = AssembledStatement::new();
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
        stmt.feed(&analysis, "gen").unwrap();
    }

    match stmt.feed(&analysis, "gen") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Genitives");
            assert_eq!(max, MAX_GENITIVES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_arrays() {
    let mut stmt = AssembledStatement::new();
    for _ in 0..MAX_ARRAYS {
        stmt.feed_array(vec![]).unwrap();
    }

    match stmt.feed_array(vec![]) {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Arrays");
            assert_eq!(max, MAX_ARRAYS);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_index_accesses() {
    let mut stmt = AssembledStatement::new();
    let array = Expr::NumberLiteral(1);
    let index = Expr::NumberLiteral(0);

    for _ in 0..MAX_INDEX_ACCESSES {
        stmt.feed_index_access(array.clone(), index.clone()).unwrap();
    }

    match stmt.feed_index_access(array, index) {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Index Accesses");
            assert_eq!(max, MAX_INDEX_ACCESSES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_nested_phrases() {
    let mut stmt = AssembledStatement::new();
    for _ in 0..MAX_NESTED_PHRASES {
        stmt.feed_nested_phrase(vec![]).unwrap();
    }

    match stmt.feed_nested_phrase(vec![]) {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Nested Phrases");
            assert_eq!(max, MAX_NESTED_PHRASES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_unwraps() {
    let mut stmt = AssembledStatement::new();
    let expr = Expr::Word(Word::new("x"));

    for _ in 0..MAX_UNWRAPS {
        stmt.feed_unwrap(expr.clone()).unwrap();
    }

    match stmt.feed_unwrap(expr) {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Unwraps");
            assert_eq!(max, MAX_UNWRAPS);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_participles() {
    let mut stmt = AssembledStatement::new();
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
        stmt.feed_participle(&analysis, "part").unwrap();
    }

    match stmt.feed_participle(&analysis, "part") {
        Err(AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Participles");
            assert_eq!(max, MAX_PARTICIPLES);
        }
        res => panic!("Expected LimitExceeded, got {:?}", res),
    }
}

#[test]
fn test_limit_blocks() {
    let mut stmt = AssembledStatement::new();
    for _ in 0..MAX_BLOCKS {
        stmt.feed_block(vec![]).unwrap();
    }

    match stmt.feed_block(vec![]) {
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
    let mut stmt = AssembledStatement::new();
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
        stmt.feed(&analysis, "καί").unwrap();
    }

    match stmt.feed(&analysis, "καί") {
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
    let mut stmt = AssembledStatement::new();

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
        stmt.feed(&subj, "text").unwrap();
        // Feed property "μῆκος" (length)
        stmt.feed(&prop_analysis, "μῆκος").unwrap();
    }

    // Try one more
    stmt.feed(&subj, "text").unwrap();
    match stmt.feed(&prop_analysis, "μῆκος") {
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
    let mut stmt = AssembledStatement::new();

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
        stmt.feed(&subj, "text").unwrap();
        stmt.feed(&prop_analysis, "μῆκος").unwrap();
    }

    // Now try to trigger a string method which would add another property access
    stmt.feed(&subj, "text").unwrap();
    stmt.feed(&delimiter_prep, "κατά").unwrap();
    stmt.feed_string(",".to_string()).unwrap(); // Delimiter literal

    // Feed split verb - should fail to create method call (return false)
    match stmt.feed(&split_verb, "σχίζεται") {
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
    let mut stmt = AssembledStatement::new();
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
        stmt.feed(&analysis, "ἤ").unwrap();
    }

    match stmt.feed(&analysis, "ἤ") {
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
    let mut stmt = AssembledStatement::new();
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
        stmt.feed(&analysis, "μεῖζον").unwrap();
    }

    match stmt.feed(&analysis, "μεῖζον") {
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
    let mut stmt = AssembledStatement::new();
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
        stmt.feed(&analysis, "ἄθροισμα").unwrap();
    }

    match stmt.feed(&analysis, "ἄθροισμα") {
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
    let mut stmt = AssembledStatement::new();

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
        stmt.feed(&subj, "text").unwrap();
        stmt.feed(&ordinal, "πρῶτον").unwrap();
    }

    stmt.feed(&subj, "text").unwrap();
    match stmt.feed(&ordinal, "πρῶτον") {
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
    let mut stmt = AssembledStatement::new();

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
    stmt.feed(&dat1, "ἀνθρώπῳ").unwrap();

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

    let result = stmt.feed(&dat2, "θεῷ");

    // SENTRY: This assertion enforces that we DO NOT allow silent overwrites.
    assert!(
        matches!(result, Err(AssemblyError::DoubleIndirect)),
        "Should return DoubleIndirect error, got {:?}",
        result
    );
}

#[test]
fn test_neuter_plural_agreement() {
    let mut stmt = AssembledStatement::new();

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
    stmt.feed(&subj, "ζῷα").unwrap();

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
    stmt.feed(&verb, "τρέχουσιν").unwrap();

    let stmt = stmt.finalize();
    assert!(
        stmt.is_ok(),
        "Neuter plural subject should allow plural verb (current behavior), got {:?}",
        stmt.err()
    );
}

#[test]
fn test_verbless_statement() {
    let mut stmt = AssembledStatement::new();

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
    stmt.feed(&subj, "ἄνθρωπος").unwrap();

    // No verb fed.

    let stmt = stmt.finalize();
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
    use crate::semantic::assembly::AssembledStatement;
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
    let mut stmt = AssembledStatement::new();

    // "μετά" (mutable marker)
    let meta = analyze("μετα"); // Preposition
    stmt.feed(&meta, "μετά").unwrap();
    let mut stmt = stmt.clone();
    assert!(stmt.has_mutable_marker);

    // "ἐν" (containment)
    let en = analyze("εν"); // Preposition
    stmt.feed(&en, "ἐν").unwrap();
    let mut stmt2 = stmt.clone();
    assert!(stmt2.has_containment_preposition);

    // "κατά" (delimiter)
    let kata = analyze("κατα"); // Preposition
    stmt.feed(&kata, "κατά").unwrap();
    let stmt3 = stmt.clone().finalize().unwrap();
    assert!(stmt3.has_delimiter_preposition);
}

#[test]
fn test_assembler_method_verbs_join_coverage() {
    let mut stmt = AssembledStatement::new();

    // 1. Subject: "list"
    let list = analyze("λιστη");
    stmt.feed(&list, "λίστη").unwrap();

    // 2. Delimiter Preposition: "κατά"
    let kata = analyze("κατα");
    stmt.feed(&kata, "κατά").unwrap();

    // 3. Delimiter Literal: ","
    stmt.feed_string(",".to_string()).unwrap();

    // 4. Join Verb: "ἑνοῦνται"
    let join = analyze("ενουνται");
    stmt.feed(&join, "ἑνοῦνται").unwrap();

    let mut stmt = stmt.clone();
    assert_eq!(
        stmt.string_method,
        Some(("join".to_string(), ",".to_string()))
    );
}

#[test]
fn test_assembler_arithmetic_operators_coverage() {
    let mut stmt = AssembledStatement::new();

    // "ἄθροισμα" (sum -> +)
    let sum = analyze("αθροισμα");
    stmt.feed(&sum, "ἄθροισμα").unwrap();

    let mut stmt = stmt.clone();
    assert!(!stmt.operators.is_empty());
}

#[test]
fn test_assembler_boolean_and_coverage() {
    let mut stmt = AssembledStatement::new();

    // "καί" (and)
    let and = analyze("και");
    stmt.feed(&and, "καί").unwrap();

    let mut stmt = stmt.clone();
    assert!(!stmt.operators.is_empty());
}

#[test]
fn test_assembler_numeral_coverage() {
    let mut stmt = AssembledStatement::new();

    // "πέντε" (5)
    let five = analyze("πεντε");
    stmt.feed(&five, "πέντε").unwrap();

    let mut stmt = stmt.clone();
    assert_eq!(stmt.literals.len(), 1);
}

#[test]
fn test_assembler_set_flags_coverage() {
    let mut stmt = AssembledStatement::new();
    stmt.set_query(true);
    stmt.set_propagate(true);

    let mut stmt = stmt.clone();
    assert!(stmt.is_query);
    assert!(stmt.is_propagate);
}

// test_assembler_has_content_coverage removed as has_content was removed

#[test]
fn test_assembler_method_verbs_split_coverage() {
    let mut stmt = AssembledStatement::new();

    // 1. Subject: "string"
    let string_noun = analyze("λογος");
    stmt.feed(&string_noun, "λόγος").unwrap();

    // 2. Delimiter Preposition: "κατά"
    let kata = analyze("κατα");
    stmt.feed(&kata, "κατά").unwrap();

    // 3. Delimiter Literal: "."
    stmt.feed_string(".".to_string()).unwrap();

    // 4. Split Verb: "σχίζεται"
    let split = analyze("σχιζεται");
    stmt.feed(&split, "σχίζεται").unwrap();

    let mut stmt = stmt.clone();
    assert_eq!(
        stmt.string_method,
        Some(("split".to_string(), ".".to_string()))
    );
}

#[test]
fn test_assembler_ordinal_index_coverage() {
    let mut stmt = AssembledStatement::new();

    // 1. Subject: "array" (use known noun "λιστη")
    let array = analyze("λιστη");
    stmt.feed(&array, "λίστη").unwrap();

    // 2. Ordinal: "first" (should map to index 0)
    let first = analyze("πρωτον");
    stmt.feed(&first, "πρῶτον").unwrap();

    let mut stmt = stmt.clone();
    assert_eq!(stmt.index_accesses.len(), 1);
    // Subject should be consumed
    assert!(stmt.subject.is_none());
}

#[test]
fn test_assembler_error_cases_coverage() {
    let mut stmt = AssembledStatement::new();

    // Double Verb
    let verb1 = analyze("λεγει");
    stmt.feed(&verb1, "λέγει").unwrap();
    let result = stmt.feed(&verb1, "λέγει");
    assert!(matches!(result, Err(AssemblyError::DoubleVerb)));
    let _ = stmt.finalize();

    // Double Object
    let obj = analyze("λογον");
    stmt.feed(&obj, "λόγον").unwrap();
    let result = stmt.feed(&obj, "λόγον");
    assert!(matches!(result, Err(AssemblyError::DoubleObject)));
    let _ = stmt.finalize();

    // Double Indirect
    let ind = analyze("ανθρωπω");
    stmt.feed(&ind, "ἀνθρώπῳ").unwrap();
    let result = stmt.feed(&ind, "ἀνθρώπῳ");
    assert!(matches!(result, Err(AssemblyError::DoubleIndirect)));
}

#[test]
fn test_constituent_derive_coverage() {
    use crate::morphology::Case;
    use crate::semantic::assembly::Constituent;
    use smol_str::SmolStr;

    let c = Constituent {
        lemma: SmolStr::new("test"),
        original: SmolStr::new("test"),
        normalized: SmolStr::new("test"),
        case: Case::Nominative,
        number: None,
        gender: None,
        person: None,
    };

    let cloned = c.clone();
    let debug = format!("{:?}", cloned);
    assert!(debug.contains("test"));
}
