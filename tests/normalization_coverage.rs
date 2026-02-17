use glossa::morphology::analyze;
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

    // We can't access conversion directly easily as it requires scope,
    // but we can verify the constituent has correct normalized form
    let subj_const = stmt.subject.as_ref().unwrap();
    assert_eq!(subj_const.normalized, "αθηναι");
    assert_eq!(subj_const.original, "Ἀθῆναι");
}

#[test]
fn test_print_normalization() {
    let mut asm = Assembler::new();

    // Print verb
    let verb = analyze("λέγε");
    asm.feed(&verb, "λέγε").unwrap();

    // Subject with diacritics
    let subj = analyze("κόσμος");
    asm.feed(&subj, "κόσμος").unwrap();

    let stmt = asm.finalize().unwrap();

    let subj_const = stmt.subject.as_ref().unwrap();
    assert_eq!(subj_const.normalized, "κοσμος");
}

#[test]
fn test_assignment_normalization() {
    let mut asm = Assembler::new();

    // Subject
    let subj = analyze("τιμή");
    asm.feed(&subj, "τιμή").unwrap();

    // Assignment verb
    let verb = analyze("γίγνεται");
    asm.feed(&verb, "γίγνεται").unwrap();

    // Value
    asm.feed_number(100).unwrap();

    let stmt = asm.finalize().unwrap();

    let subj_const = stmt.subject.as_ref().unwrap();
    assert_eq!(subj_const.normalized, "τιμη");
}

#[test]
fn test_participle_normalization_in_binding() {
    let mut asm = Assembler::new();

    // Participle as subject (implied)
    let part_analysis = glossa::morphology::ParticipleAnalysis {
        stem: "λεγο".to_string(),
        tense: glossa::morphology::Tense::Present,
        voice: glossa::morphology::Voice::Active,
        case: glossa::morphology::Case::Nominative,
        gender: glossa::morphology::Gender::Neuter,
        number: glossa::morphology::Number::Singular,
        confidence: 1.0,
    };

    // Feed with diacritics
    asm.feed_participle(&part_analysis, "λεγόμενον", "λεγομενον")
        .unwrap();

    // Binding verb
    let verb = analyze("ἔστω");
    asm.feed(&verb, "ἔστω").unwrap();

    // Value
    asm.feed_number(1).unwrap();

    let stmt = asm.finalize().unwrap();

    // Check participle list
    assert!(!stmt.participles.is_empty());
    assert_eq!(stmt.participles[0].normalized, "λεγομενον");
}

#[test]
fn test_adjective_normalization() {
    let mut asm = Assembler::new();

    // Adjective "καλός" (good)
    let adj = analyze("καλός");
    asm.feed(&adj, "καλός").unwrap();

    let stmt = asm.finalize().unwrap();

    assert!(!stmt.adjectives.is_empty());
    assert_eq!(stmt.adjectives[0].normalized, "καλος");
}

#[test]
fn test_collection_ops_normalization() {
    let mut asm = Assembler::new();

    // Subject "λίστη" (list)
    let subj = analyze("λίστη");
    asm.feed(&subj, "λίστη").unwrap();

    // Push verb "ὠθεῖ" (pushes)
    let verb = analyze("ὠθεῖ");
    asm.feed(&verb, "ὠθεῖ").unwrap();

    // Value to push
    asm.feed_number(5).unwrap();

    let stmt = asm.finalize().unwrap();

    let subj_const = stmt.subject.as_ref().unwrap();
    assert_eq!(subj_const.normalized, "λιστη");

    // Also verify verb normalized form
    let verb_const = stmt.verb.as_ref().unwrap();
    assert_eq!(verb_const.normalized, "ωθει");
}

#[test]
fn test_assertion_normalization() {
    let mut asm = Assembler::new();

    // "χ" (chi - variable)
    let subj = analyze("χ");
    asm.feed(&subj, "χ").unwrap();

    // "ἐν" (in - containment) - triggers assertion logic
    let en = analyze("ἐν");
    asm.feed(&en, "ἐν").unwrap();

    // "λίστῃ" (list - dative)
    let _collection = analyze("λίστῃ"); // normalized "λιστη"
    // Use feed_with_normalized to simulate context where it might be recognized as variable
    // Usually collection goes to different slot, but for "element in collection",
    // Assembler puts "element" as subject?
    // Wait, "2 ἐν χ δεῖ" -> Subject is 2? No, subject slot.
    // Let's use "χ ἐν ψ δεῖ".
    // "χ" (Nom) -> Subject.
    // "ἐν" (Prep) -> flag.
    // "ψ" -> Object? Or part of prep phrase?
    // Assembler doesn't have prep slots, usually handles this via flags or special handling.
    // Let's look at `classify_assertion` in conversion.rs:
    // `if asm_stmt.has_containment_preposition && let Some(ref subj) = asm_stmt.subject`
    // It treats subject as collection!
    // "2 ἐν χ δεῖ" -> "2" is literal. "χ" is subject?
    // If word order is "2 ἐν χ δεῖ":
    // Feed 2 (literal). Feed ἐν (flag). Feed χ (Nom/Dat/Acc). Feed δεῖ (Verb).
    // If χ is Nom, it goes to Subject.
    // So "2 in X must" -> Subject=X.

    // Subject "χάρτης" (map)
    let map = analyze("χάρτης");
    asm.feed(&map, "χάρτης").unwrap();

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

    // "τιμή" (value)
    let subj = analyze("τιμή");
    asm.feed(&subj, "τιμή").unwrap();

    // "ἰσοῦται" (equals)
    let verb = analyze("ἰσοῦται");
    asm.feed(&verb, "ἰσοῦται").unwrap();

    // 5
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

    // "κόσμος" (world)
    let subj = analyze("κόσμος");
    asm.feed(&subj, "κόσμος").unwrap();

    asm.set_query(true);

    let stmt = asm.finalize().unwrap();

    let subj_const = stmt.subject.as_ref().unwrap();
    assert_eq!(subj_const.normalized, "κοσμος");
}
