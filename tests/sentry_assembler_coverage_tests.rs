#![allow(missing_docs)]
use glossa::morphology::BinaryOp;
use glossa::morphology::analyze;
use glossa::morphology::{Case, Gender, MorphAnalysis, Number, PartOfSpeech, Person};
use glossa::semantic::Literal;
use glossa::semantic::{Assembler, AssemblyError};
use std::borrow::Cow;

#[test]
fn test_assembler_comparison_operator() {
    let mut asm = Assembler::new();
    let analysis = analyze("ἴσον"); // "equal to"
    asm.feed(&analysis, "ἴσον").unwrap();
    let final_stmt = asm.finalize().unwrap();
    assert_eq!(final_stmt.operators, vec![BinaryOp::Eq]);
}

#[test]
fn test_assembler_arithmetic_operator() {
    let mut asm = Assembler::new();
    let analysis = analyze("ἄθροισμα"); // "sum"
    asm.feed(&analysis, "ἄθροισμα").unwrap();
    let final_stmt = asm.finalize().unwrap();
    assert_eq!(final_stmt.operators, vec![BinaryOp::Add]);
}

#[test]
fn test_assembler_numeral_value() {
    let mut asm = Assembler::new();
    let analysis = analyze("πέντε"); // "five"
    asm.feed(&analysis, "πέντε").unwrap();
    let final_stmt = asm.finalize().unwrap();
    assert_eq!(final_stmt.literals.len(), 1);
    if let Literal::Number(n) = final_stmt.literals[0] {
        assert_eq!(n, 5);
    } else {
        panic!("Expected Number literal");
    }
}

fn make_analysis(
    lemma: &str,
    pos: PartOfSpeech,
    case: Option<Case>,
    number: Option<Number>,
) -> MorphAnalysis {
    MorphAnalysis {
        lemma: Cow::Owned(lemma.to_string()),
        part_of_speech: pos,
        case,
        number,
        gender: Some(Gender::Masculine),
        person: None,
        tense: None,
        voice: None,
        mood: None,
        confidence: 1.0,
    }
}

#[test]
fn test_handle_unknown_case_double_object() {
    let mut asm = Assembler::new();
    let obj1 = make_analysis("obj1", PartOfSpeech::Noun, None, Some(Number::Singular));
    asm.feed(&obj1, "obj1").unwrap();
    let obj2 = make_analysis("obj2", PartOfSpeech::Noun, None, Some(Number::Singular));
    let res = asm.feed(&obj2, "obj2");
    assert!(matches!(res, Err(AssemblyError::DoubleObject)));
}

#[test]
fn test_handle_dative_double_indirect() {
    let mut asm = Assembler::new();
    let dat1 = make_analysis(
        "dat1",
        PartOfSpeech::Noun,
        Some(Case::Dative),
        Some(Number::Singular),
    );
    asm.feed(&dat1, "dat1").unwrap();
    let dat2 = make_analysis(
        "dat2",
        PartOfSpeech::Noun,
        Some(Case::Dative),
        Some(Number::Singular),
    );
    let res = asm.feed(&dat2, "dat2");
    assert!(matches!(res, Err(AssemblyError::DoubleIndirect)));
}

#[test]
fn test_handle_vocative_subject_already_exists() {
    let mut asm = Assembler::new();
    let subj = make_analysis(
        "subj",
        PartOfSpeech::Noun,
        Some(Case::Nominative),
        Some(Number::Singular),
    );
    asm.feed(&subj, "subj").unwrap();
    let voc = make_analysis(
        "voc",
        PartOfSpeech::Noun,
        Some(Case::Vocative),
        Some(Number::Singular),
    );
    asm.feed(&voc, "voc").unwrap();
    let mut verb_analysis = make_analysis("verb", PartOfSpeech::Verb, None, Some(Number::Singular));
    verb_analysis.person = Some(Person::Third);
    asm.feed(&verb_analysis, "verb").unwrap();
    let stmt = asm.finalize().unwrap();
    assert_eq!(stmt.subject.unwrap().original, "subj");
}

#[test]
fn test_handle_accusative_double_object() {
    let mut asm = Assembler::new();
    let acc1 = make_analysis(
        "acc1",
        PartOfSpeech::Noun,
        Some(Case::Accusative),
        Some(Number::Singular),
    );
    asm.feed(&acc1, "acc1").unwrap();
    let acc2 = make_analysis(
        "acc2",
        PartOfSpeech::Noun,
        Some(Case::Accusative),
        Some(Number::Singular),
    );
    let res = asm.feed(&acc2, "acc2");
    assert!(matches!(res, Err(AssemblyError::DoubleObject)));
}

#[test]
fn test_handle_verb_double_verb() {
    let mut asm = Assembler::new();
    let verb1 = make_analysis("verb1", PartOfSpeech::Verb, None, Some(Number::Singular));
    asm.feed(&verb1, "verb1").unwrap();
    let verb2 = make_analysis("verb2", PartOfSpeech::Verb, None, Some(Number::Singular));
    let res = asm.feed(&verb2, "verb2");
    assert!(matches!(res, Err(AssemblyError::DoubleVerb)));
}

#[test]
fn test_handle_subject_double_subject() {
    let mut asm = Assembler::new();
    let subj1 = make_analysis(
        "subj1",
        PartOfSpeech::Noun,
        Some(Case::Nominative),
        Some(Number::Singular),
    );
    asm.feed(&subj1, "subj1").unwrap();
    let subj2 = make_analysis(
        "subj2",
        PartOfSpeech::Noun,
        Some(Case::Nominative),
        Some(Number::Singular),
    );
    asm.feed(&subj2, "subj2").unwrap();
    let mut verb_analysis = make_analysis("verb", PartOfSpeech::Verb, None, Some(Number::Singular));
    verb_analysis.person = Some(Person::Third);
    asm.feed(&verb_analysis, "verb").unwrap();
    let res = asm.finalize();
    assert!(matches!(res, Err(AssemblyError::DoubleSubject)));
}

#[test]
fn test_check_agreement_neuter_plural_takes_singular_verb() {
    let mut asm = Assembler::new();
    let mut subj = make_analysis(
        "neuter_plural",
        PartOfSpeech::Noun,
        Some(Case::Nominative),
        Some(Number::Plural),
    );
    subj.gender = Some(Gender::Neuter);
    asm.feed(&subj, "neuter_plural").unwrap();
    let mut verb = make_analysis("verb", PartOfSpeech::Verb, None, Some(Number::Singular));
    verb.person = Some(Person::Third);
    asm.feed(&verb, "verb").unwrap();
    let stmt = asm.finalize().unwrap();
    assert_eq!(stmt.subject.unwrap().original, "neuter_plural");
}

#[test]
fn test_check_agreement_disagreement_number() {
    let mut asm = Assembler::new();
    let mut subj = make_analysis(
        "masc_plural",
        PartOfSpeech::Noun,
        Some(Case::Nominative),
        Some(Number::Plural),
    );
    subj.gender = Some(Gender::Masculine);
    asm.feed(&subj, "masc_plural").unwrap();
    let mut verb = make_analysis("verb", PartOfSpeech::Verb, None, Some(Number::Singular));
    verb.person = Some(Person::Third);
    let res = asm.feed(&verb, "verb");
    assert!(matches!(
        res,
        Err(AssemblyError::SubjectVerbDisagreement { .. })
    ));
}

#[test]
fn test_check_agreement_disagreement_person() {
    let mut asm = Assembler::new();
    let mut subj = make_analysis(
        "subj",
        PartOfSpeech::Noun,
        Some(Case::Nominative),
        Some(Number::Singular),
    );
    subj.person = Some(Person::First);
    asm.feed(&subj, "subj").unwrap();
    let mut verb = make_analysis("verb", PartOfSpeech::Verb, None, Some(Number::Singular));
    verb.person = Some(Person::Third);
    let res = asm.feed(&verb, "verb");
    assert!(matches!(
        res,
        Err(AssemblyError::SubjectVerbDisagreement { .. })
    ));
}

#[test]
fn test_handle_nominative_agreement_check() {
    let mut asm = Assembler::new();
    let mut verb = make_analysis("verb", PartOfSpeech::Verb, None, Some(Number::Singular));
    verb.person = Some(Person::Third);
    asm.feed(&verb, "verb").unwrap();
    let mut subj = make_analysis(
        "subj",
        PartOfSpeech::Noun,
        Some(Case::Nominative),
        Some(Number::Plural),
    );
    subj.person = Some(Person::Third);
    let res = asm.feed(&subj, "subj");
    assert!(matches!(
        res,
        Err(AssemblyError::SubjectVerbDisagreement { .. })
    ));
}

#[test]
fn test_handle_genitive_limit_exceeded() {
    let mut asm = Assembler::new();
    let genitive_word = make_analysis(
        "gen",
        PartOfSpeech::Noun,
        Some(Case::Genitive),
        Some(Number::Singular),
    );
    let mut max_hit = false;
    for _ in 0..1024 {
        if let Err(AssemblyError::LimitExceeded { .. }) = asm.feed(&genitive_word, "gen") {
            max_hit = true;
            break;
        }
    }
    assert!(max_hit, "Should have hit MAX_GENITIVES limit");
}

#[test]
fn test_missing_verb_limit_branch() {
    let mut asm = Assembler::new();
    asm.feed_number(42).unwrap();
    let stmt = asm.finalize().unwrap();
    assert!(stmt.verb.is_none());
    assert_eq!(stmt.literals.len(), 1);
}
