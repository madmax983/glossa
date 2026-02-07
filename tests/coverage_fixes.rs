use glossa::parser::parse;
use glossa::semantic::{analyze_program, Assembler};
use glossa::morphology::{MorphAnalysis, PartOfSpeech, Case, Number, Gender, Person};
use std::borrow::Cow;

/// Helper to compile GLOSSA source
#[allow(dead_code)]
fn compile(source: &str) -> String {
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    glossa::codegen::generate_rust(&analyzed)
}

/// Helper to compile GLOSSA source expecting an error
fn compile_error(source: &str) -> String {
    let ast = parse(source).unwrap();
    analyze_program(&ast).unwrap_err().to_string()
}

// ============================================================================
// Assembler Fallback Tests
// ============================================================================

#[test]
fn test_assembler_unknown_fallback_to_subject() {
    // "foobar" is unknown, should become Subject
    let mut asm = Assembler::new();
    let unknown = MorphAnalysis {
        lemma: Cow::Borrowed("foobar"),
        part_of_speech: PartOfSpeech::Unknown,
        case: None,
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 0.0,
    };

    asm.feed(&unknown, "foobar").unwrap();
    let stmt = asm.finalize().unwrap();

    assert!(stmt.subject.is_some());
    assert_eq!(stmt.subject.unwrap().original, "foobar");
}

#[test]
fn test_assembler_unknown_fallback_to_object() {
    // "user" (Subject) "foobar" (unknown -> Object)
    let mut asm = Assembler::new();

    // Feed Subject
    let subj = MorphAnalysis {
        lemma: Cow::Borrowed("user"),
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
    asm.feed(&subj, "user").unwrap();

    // Feed Unknown
    let unknown = MorphAnalysis {
        lemma: Cow::Borrowed("foobar"),
        part_of_speech: PartOfSpeech::Unknown,
        case: None,
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 0.0,
    };
    asm.feed(&unknown, "foobar").unwrap();

    let stmt = asm.finalize().unwrap();

    assert!(stmt.subject.is_some());
    assert!(stmt.object.is_some());
    assert_eq!(stmt.object.unwrap().original, "foobar");
}

#[test]
fn test_assembler_unknown_fallback_to_nominative() {
    // "user" (Subject) "data" (Object) "foobar" (unknown -> Nominative)
    let mut asm = Assembler::new();

    // Feed Subject
    let subj = MorphAnalysis {
        lemma: Cow::Borrowed("user"),
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
    asm.feed(&subj, "user").unwrap();

    // Feed Object
    let obj = MorphAnalysis {
        lemma: Cow::Borrowed("data"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Accusative),
        number: Some(Number::Singular),
        gender: Some(Gender::Neuter),
        person: Some(Person::Third),
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };
    asm.feed(&obj, "data").unwrap();

    // Feed Unknown
    let unknown = MorphAnalysis {
        lemma: Cow::Borrowed("foobar"),
        part_of_speech: PartOfSpeech::Unknown,
        case: None,
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 0.0,
    };
    asm.feed(&unknown, "foobar").unwrap();

    let stmt = asm.finalize().unwrap();

    assert!(stmt.subject.is_some());
    assert!(stmt.object.is_some());
    assert_eq!(stmt.nominatives.len(), 1);
    assert_eq!(stmt.nominatives[0].original, "foobar");
}

#[test]
fn test_assembler_ambiguity_hen_vs_en() {
    let mut asm = Assembler::new();

    // Feed "ἓν" (one) with rough breathing (U+1F13, normalized to εν with rough breathing checked in original)
    // The normalized form is "εν".
    // "ἓν" (U+1F13) decomposes to ε (U+03B5) + rough breathing (U+0314) + grave (U+0300)
    // We need to construct a string that `check_special_markers` sees as having rough breathing.

    let hen = "ἓν"; // 1F13 (includes rough breathing)

    // This analysis would normally look like a preposition if we only checked "εν"
    let analysis = MorphAnalysis {
        lemma: Cow::Borrowed("εν"),
        part_of_speech: PartOfSpeech::Preposition,
        case: None,
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 0.5,
    };

    asm.feed(&analysis, hen).unwrap();
    let stmt = asm.finalize().unwrap();

    // It should NOT set has_containment_preposition
    assert!(!stmt.has_containment_preposition);
}

// ============================================================================
// extract_value Error Tests
// ============================================================================

#[test]
fn test_extract_value_unable_to_determine() {
    // Statement with content but no valid value extraction
    // "α" (variable, but no verb, no binding) -> Expression statement
    // If "α" is undefined, extract_value handles operands but classify_expression might just wrap it.
    // We need to trigger the specific error in `extract_value` "Unable to determine value".
    // This happens if we have subject/object/literals but NO operators and NO single literal/object match.
    // Wait, extract_value has "Otherwise use object" and "Default" fallback.
    // The "Default" fallback now returns Error if `has_content` is true.
    // So if we have Subject but NO Object and NO Literals and NO Operators...
    // But `extract_value` puts Subject in `operands` if operators exist.
    // If NO operators...
    // `extract_value` checks `literals.first()`.
    // Then checks `object`.
    // If we ONLY have a Subject, it falls through to Default.

    // So "α" (Subject) alone should trigger this if "α" is defined.
    // But "α" alone is a valid statement? No, usually meaningless.
    // Let's try: "α" where α is defined.

    let source = "α 1 ἔστω. α.";
    // This might parse as "α" (Subject).
    // `classify_assembled_statement` -> `classify_expression` -> `extract_value`.
    // `extract_value`:
    // - Unwraps? No.
    // - Enum? No.
    // - Property? No.
    // - Index? No.
    // - Array? No.
    // - Operators? No.
    // - Literals? No.
    // - Object? No (it's Subject).
    // - Default -> Check has_content. Subject is Some. Error!

    let err = compile_error(source);
    assert!(err.contains("Unable to determine value"));
}

#[test]
fn test_extract_value_insufficient_operands() {
    // Binary operation with only 1 operand
    // "α ἄθροισμα" (Subject + Operator)
    let source = "α 1 ἔστω. α ἄθροισμα.";
    let err = compile_error(source);
    assert!(err.contains("Operation requires at least two operands"));
}

// ============================================================================
// classify_print Error Tests
// ============================================================================

#[test]
fn test_classify_print_undefined_subject() {
    let source = "ἀγνωστος λέγε.";
    let err = compile_error(source);
    assert!(err.contains("Undefined variable"));
    assert!(err.contains("ἀγνωστος"));
}

#[test]
fn test_classify_print_undefined_object() {
    let source = "λέγε ἀγνωστος.";
    let err = compile_error(source);
    assert!(err.contains("Undefined variable"));
    assert!(err.contains("ἀγνωστος"));
}
