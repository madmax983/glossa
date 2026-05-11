use super::*;

#[test]
fn test_lookup_verb() {
    let entry = lookup("λεγε").unwrap();
    assert_eq!(entry.pos, PartOfSpeech::Verb);
    assert_eq!(entry.mood, Some(Mood::Imperative));
}

#[test]
fn test_lookup_binding() {
    let entry = lookup("εστω").unwrap();
    assert_eq!(entry.pos, PartOfSpeech::Verb);
    assert!(is_binding_verb("εστω"));
}

#[test]
fn test_lookup_type() {
    let entry = lookup("αριθμος").unwrap();
    assert_eq!(entry.pos, PartOfSpeech::Noun);
    assert_eq!(entry.rust_equiv, Some("i64"));
}

#[test]
fn test_is_print_verb() {
    assert!(is_print_verb("λεγε"));
    assert!(is_print_verb("γραφε"));
    assert!(!is_print_verb("εστω"));
}

#[test]
fn test_numeral_value() {
    assert_eq!(numeral_value("πεντε"), Some(5));
    assert_eq!(numeral_value("δεκα"), Some(10));
    assert_eq!(numeral_value("foo"), None);
}

#[test]
fn test_boolean_lookup() {
    let entry = lookup("αληθες").unwrap();
    assert_eq!(entry.rust_equiv, Some("true"));

    let entry = lookup("ψευδος").unwrap();
    assert_eq!(entry.rust_equiv, Some("false"));
}

#[test]
fn test_comparison_operators() {
    assert_eq!(comparison_operator("μειζον"), Some(BinaryOp::Gt));
    assert_eq!(comparison_operator("ελαττον"), Some(BinaryOp::Lt));
    assert_eq!(comparison_operator("ισον"), Some(BinaryOp::Eq));
    assert_eq!(comparison_operator("ανισον"), Some(BinaryOp::Ne));
    assert_eq!(comparison_operator("foo"), None);
}

#[test]
fn test_boolean_operators() {
    assert_eq!(boolean_operator("και"), Some(BinaryOp::And));
    assert_eq!(boolean_operator("η"), Some(BinaryOp::Or));
    assert!(is_negation("ου"));
    assert!(is_negation("ουκ"));
    assert!(is_negation("ουχ"));
}

#[test]
fn test_arithmetic_operators() {
    assert_eq!(arithmetic_operator("αθροισμα"), Some(BinaryOp::Add));
    assert_eq!(arithmetic_operator("διαφορα"), Some(BinaryOp::Sub));
    assert_eq!(arithmetic_operator("γινομενον"), Some(BinaryOp::Mul));
    assert_eq!(arithmetic_operator("μερος"), Some(BinaryOp::Div));
    assert_eq!(arithmetic_operator("υπολοιπον"), Some(BinaryOp::Mod));
}

#[test]
fn test_operator_lexicon_entries() {
    // Comparison adjectives
    let entry = lookup("μειζον").unwrap();
    assert_eq!(entry.rust_equiv, Some(">"));
    assert_eq!(entry.pos, PartOfSpeech::Adjective);

    // Boolean particles
    let entry = lookup("και").unwrap();
    assert_eq!(entry.rust_equiv, Some("&&"));
    assert_eq!(entry.pos, PartOfSpeech::Conjunction);

    // Arithmetic nouns
    let entry = lookup("αθροισμα").unwrap();
    assert_eq!(entry.rust_equiv, Some("+"));
    assert_eq!(entry.pos, PartOfSpeech::Noun);
}

#[test]
fn test_meta_is_mutable_marker() {
    assert!(is_mutable_marker("μετα"));
}

#[test]
fn test_meta_lexicon_entry() {
    let entry = lookup("μετα").unwrap();
    assert_eq!(entry.pos, PartOfSpeech::Preposition);
    assert_eq!(entry.rust_equiv, Some("mut"));
}

#[test]
fn test_non_mutable_marker() {
    assert!(!is_mutable_marker("εστω"));
    assert!(!is_mutable_marker("λεγε"));
}

#[test]
fn test_gignetai_is_assignment_verb() {
    assert!(is_assignment_verb("γιγνεται"));
    assert!(is_assignment_verb("γιγνομαι"));
}

#[test]
fn test_gignetai_lexicon_entry() {
    let entry = lookup("γιγνεται").unwrap();
    assert_eq!(entry.pos, PartOfSpeech::Verb);
    assert_eq!(entry.voice, Some(Voice::Middle));
    assert_eq!(entry.lemma, "γιγνομαι");
}

#[test]
fn test_non_assignment_verb() {
    assert!(!is_assignment_verb("εστω"));
    assert!(!is_assignment_verb("λεγε"));
}

// =========================================================================
// HashMap/HashSet/String operation tests (Issue #77)
// =========================================================================

#[test]
fn test_is_insert_verb() {
    assert!(is_insert_verb("τιθησι"));
    assert!(is_insert_verb("τιθημι"));
    assert!(is_insert_verb("θες"));
    assert!(!is_insert_verb("λεγε"));
    assert!(!is_insert_verb("ωθει"));
}

#[test]
fn test_is_split_verb() {
    assert!(is_split_verb("σχιζει"));
    assert!(is_split_verb("σχιζεται"));
    assert!(is_split_verb("σχιζω"));
    assert!(!is_split_verb("λεγε"));
}

#[test]
fn test_is_join_verb() {
    assert!(is_join_verb("ενουνται"));
    assert!(is_join_verb("ενουσι"));
    assert!(is_join_verb("ενοω"));
    assert!(!is_join_verb("λεγε"));
}

#[test]
fn test_is_containment_preposition() {
    assert!(is_containment_preposition("εν"));
    assert!(!is_containment_preposition("δια"));
    assert!(!is_containment_preposition("εις"));
}

#[test]
fn test_is_delimiter_preposition() {
    assert!(is_delimiter_preposition("κατα"));
    assert!(is_delimiter_preposition("κατ"));
    assert!(!is_delimiter_preposition("εν"));
    assert!(!is_delimiter_preposition("δια"));
}

#[test]
fn test_insert_verb_lexicon_entries() {
    let entry = lookup("τιθησι").unwrap();
    assert_eq!(entry.pos, PartOfSpeech::Verb);
    assert_eq!(entry.rust_equiv, Some(".insert"));
    assert_eq!(entry.lemma, "τιθημι");

    let entry = lookup("θες").unwrap();
    assert_eq!(entry.mood, Some(Mood::Imperative));
}

#[test]
fn test_split_verb_lexicon_entries() {
    let entry = lookup("σχιζεται").unwrap();
    assert_eq!(entry.pos, PartOfSpeech::Verb);
    assert_eq!(entry.rust_equiv, Some(".split"));
    assert_eq!(entry.voice, Some(Voice::Middle));
}

#[test]
fn test_join_verb_lexicon_entries() {
    let entry = lookup("ενουνται").unwrap();
    assert_eq!(entry.pos, PartOfSpeech::Verb);
    assert_eq!(entry.rust_equiv, Some(".join"));
    assert_eq!(entry.voice, Some(Voice::Middle));
}

#[test]
fn test_declined_collection_nouns() {
    // HashMap declined forms
    let entry = lookup("χαρτη").unwrap();
    assert_eq!(entry.case, Some(Case::Dative));
    assert_eq!(entry.lemma, "χαρτης");

    let entry = lookup("χαρτου").unwrap();
    assert_eq!(entry.case, Some(Case::Genitive));

    // HashSet declined forms
    let entry = lookup("συνολω").unwrap();
    assert_eq!(entry.case, Some(Case::Dative));
    assert_eq!(entry.lemma, "συνολον");

    // String declined forms
    let entry = lookup("λογω").unwrap();
    assert_eq!(entry.case, Some(Case::Dative));
    assert_eq!(entry.lemma, "λογος");
}

#[test]
fn test_is_assert_verb() {
    assert!(is_assert_verb("δει"));
    assert!(is_assert_verb("dei"));
    assert!(!is_assert_verb("εστω"));
    assert!(!is_assert_verb("λεγε"));
}

#[test]
fn test_is_equals_verb() {
    assert!(is_equals_verb("ισοω"));
    assert!(is_equals_verb("isoo"));
    assert!(is_equals_verb("ισουται"));
    assert!(is_equals_verb("isoutai"));
    assert!(!is_equals_verb("δει"));
    assert!(!is_equals_verb("εστω"));
}

#[test]
fn test_assert_verb_lexicon_entry() {
    let entry = lookup("δει").unwrap();
    assert_eq!(entry.pos, PartOfSpeech::Verb);
    assert_eq!(entry.rust_equiv, Some("assert!"));
    assert_eq!(entry.lemma, "δει");
    assert_eq!(entry.person, Some(Person::Third));
    assert_eq!(entry.number, Some(Number::Singular));
}

#[test]
fn test_equals_verb_lexicon_entry() {
    let entry = lookup("ισουται").unwrap();
    assert_eq!(entry.pos, PartOfSpeech::Verb);
    assert_eq!(entry.rust_equiv, Some("assert_eq!"));
    assert_eq!(entry.lemma, "ισοω");
    assert_eq!(entry.voice, Some(Voice::Middle));
    assert_eq!(entry.person, Some(Person::Third));
    assert_eq!(entry.number, Some(Number::Singular));
}
