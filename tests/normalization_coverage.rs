use glossa::morphology::{Case, Gender, MorphAnalysis, Number, PartOfSpeech, analyze};
use glossa::semantic::Assembler;

#[test]
fn test_binding_normalization() {
    let mut asm = Assembler::new();

    // Test that "Ἀθῆναι" (Athena) binds as "αθηναι"
    let mut subj = analyze("Ἀθῆναι");
    subj.number = Some(glossa::morphology::Number::Singular); // Ensure singular for agreement
    asm.feed(&subj, "Ἀθῆναι").unwrap();

    // Binding verb
    let verb = analyze("ἔστω");
    asm.feed(&verb, "ἔστω").unwrap();

    // Value
    asm.feed_number(42).unwrap();

    let stmt = asm.finalize().unwrap();

    let subj_const = stmt.subject.as_ref().unwrap();
    assert_eq!(subj_const.normalized, "αθηναι");
    assert_eq!(subj_const.original, "Ἀθῆναι");
}

#[test]
fn test_print_normalization() {
    let mut asm = Assembler::new();

    let verb = analyze("λέγε");
    asm.feed(&verb, "λέγε").unwrap();

    let subj = analyze("κόσμος");
    asm.feed(&subj, "κόσμος").unwrap();

    let stmt = asm.finalize().unwrap();

    let subj_const = stmt.subject.as_ref().unwrap();
    assert_eq!(subj_const.normalized, "κοσμος");
}

#[test]
fn test_assignment_normalization() {
    let mut asm = Assembler::new();

    let subj = analyze("τιμή");
    asm.feed(&subj, "τιμή").unwrap();

    let verb = analyze("γίγνεται");
    asm.feed(&verb, "γίγνεται").unwrap();

    asm.feed_number(100).unwrap();

    let stmt = asm.finalize().unwrap();

    let subj_const = stmt.subject.as_ref().unwrap();
    assert_eq!(subj_const.normalized, "τιμη");
}

#[test]
fn test_participle_normalization_in_binding() {
    let mut asm = Assembler::new();

    let part_analysis = glossa::morphology::ParticipleAnalysis {
        stem: "λεγο".to_string(),
        tense: glossa::morphology::Tense::Present,
        voice: glossa::morphology::Voice::Active,
        case: glossa::morphology::Case::Nominative,
        gender: glossa::morphology::Gender::Neuter,
        number: glossa::morphology::Number::Singular,
        confidence: 1.0,
    };

    asm.feed_participle(&part_analysis, "λεγόμενον", "λεγομενον")
        .unwrap();

    let verb = analyze("ἔστω");
    asm.feed(&verb, "ἔστω").unwrap();

    asm.feed_number(1).unwrap();

    let stmt = asm.finalize().unwrap();

    assert!(!stmt.participles.is_empty());
    assert_eq!(stmt.participles[0].normalized, "λεγομενον");
}

#[test]
fn test_adjective_normalization() {
    let mut asm = Assembler::new();

    // Manually construct analysis to ensure it's treated as Adjective
    // "καλός" might be analyzed as Noun in some contexts in lexicon
    let adj = MorphAnalysis {
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
    asm.feed(&adj, "καλός").unwrap();

    let verb = analyze("λέγε");
    asm.feed(&verb, "λέγε").unwrap();

    let stmt = asm.finalize().unwrap();

    assert!(
        !stmt.adjectives.is_empty(),
        "Adjectives list should not be empty"
    );
    assert_eq!(stmt.adjectives[0].normalized, "καλος");
}

#[test]
fn test_collection_ops_normalization() {
    let mut asm = Assembler::new();

    let subj = analyze("λίστη");
    asm.feed(&subj, "λίστη").unwrap();

    let verb = analyze("ὠθεῖ");
    asm.feed(&verb, "ὠθεῖ").unwrap();

    asm.feed_number(5).unwrap();

    let stmt = asm.finalize().unwrap();

    let subj_const = stmt.subject.as_ref().unwrap();
    assert_eq!(subj_const.normalized, "λιστη");

    let verb_const = stmt.verb.as_ref().unwrap();
    assert_eq!(verb_const.normalized, "ωθει");
}

#[test]
fn test_assertion_normalization() {
    let mut asm = Assembler::new();

    // Swap order: Feed Collection first so it becomes Subject
    // conversion.rs expects Subject to be the Collection

    // Subject "χάρτης" (map) - Nominative
    let map = analyze("χάρτης");
    asm.feed(&map, "χάρτης").unwrap();

    // "ἐν" (in - containment)
    let en = analyze("ἐν");
    asm.feed(&en, "ἐν").unwrap();

    // "χ" (chi - variable) - Feed as literal string/variable or just feed it?
    // If we feed "χ" as MorphAnalysis(Nominative), it will go to `nominatives` since Subject is full.
    // If conversion.rs looks for element in literals or object...
    // Let's assume we are testing "χάρτης" normalization primarily here.
    // We can feed "2" as literal to satisfy the element requirement of `classify_assertion`
    asm.feed_number(2).unwrap();

    // "δεῖ" (must)
    let verb = analyze("δεῖ");
    asm.feed(&verb, "δεῖ").unwrap();

    let stmt = asm.finalize().unwrap();

    let subj_const = stmt.subject.as_ref().unwrap();
    assert_eq!(subj_const.normalized, "χαρτης");
    assert!(stmt.has_containment_preposition);
}

#[test]
fn test_equality_normalization() {
    let mut asm = Assembler::new();

    let subj = analyze("τιμή");
    asm.feed(&subj, "τιμή").unwrap();

    let verb = analyze("ἰσοῦται");
    asm.feed(&verb, "ἰσοῦται").unwrap();

    asm.feed_number(5).unwrap();

    let stmt = asm.finalize().unwrap();

    let subj_const = stmt.subject.as_ref().unwrap();
    assert_eq!(subj_const.normalized, "τιμη");

    let verb_const = stmt.verb.as_ref().unwrap();
    assert_eq!(verb_const.normalized, "ισουται");
}

#[test]
fn test_query_normalization() {
    let mut asm = Assembler::new();

    let subj = analyze("κόσμος");
    asm.feed(&subj, "κόσμος").unwrap();

    asm.set_query(true);

    let stmt = asm.finalize().unwrap();

    let subj_const = stmt.subject.as_ref().unwrap();
    assert_eq!(subj_const.normalized, "κοσμος");
}
