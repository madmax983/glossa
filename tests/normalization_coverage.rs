use glossa::morphology::analyze;
use glossa::semantic::{Assembler, Constituent};

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
    asm.feed_participle(&part_analysis, "λεγόμενον", "λεγομενον").unwrap();

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
