#![allow(missing_docs)]
use glossa::morphology::models::*;

#[test]
fn test_morph_analysis_creation() {
    let analysis = MorphAnalysis::new("λογος".to_string(), PartOfSpeech::Noun);
    assert_eq!(analysis.lemma, "λογος");
    assert_eq!(analysis.part_of_speech, PartOfSpeech::Noun);
    assert_eq!(analysis.confidence, 0.5); // Default confidence
}

#[test]
fn test_morph_analysis_with_confidence() {
    let analysis = MorphAnalysis::new("λογος".to_string(), PartOfSpeech::Noun).with_confidence(0.9);
    assert_eq!(analysis.confidence, 0.9);
}

#[test]
fn test_analyses_compatible() {
    // Compatible: Same case
    let mut a = MorphAnalysis::new("λογος".to_string(), PartOfSpeech::Noun);
    a.case = Some(Case::Nominative);
    let mut b = MorphAnalysis::new("λογος".to_string(), PartOfSpeech::Noun);
    b.case = Some(Case::Nominative);
    assert!(analyses_compatible(&a, &b));

    // Incompatible: Different case
    b.case = Some(Case::Accusative);
    assert!(!analyses_compatible(&a, &b));

    // Compatible: One has no case
    b.case = None;
    assert!(analyses_compatible(&a, &b));

    // Incompatible: Different number
    a.number = Some(Number::Singular);
    b.number = Some(Number::Plural);
    assert!(!analyses_compatible(&a, &b));

    // Incompatible: Different gender
    a.number = None;
    b.number = None;
    a.gender = Some(Gender::Masculine);
    b.gender = Some(Gender::Feminine);
    assert!(!analyses_compatible(&a, &b));
}

#[test]
fn test_display_impls() {
    assert_eq!(Case::Nominative.to_string(), "ὀνομαστική");
    assert_eq!(Case::Genitive.to_string(), "γενική");
    assert_eq!(Case::Dative.to_string(), "δοτική");
    assert_eq!(Case::Accusative.to_string(), "αἰτιατική");
    assert_eq!(Case::Vocative.to_string(), "κλητική");

    assert_eq!(Number::Singular.to_string(), "ἑνικός");
    assert_eq!(Number::Plural.to_string(), "πληθυντικός");

    assert_eq!(Gender::Masculine.to_string(), "ἀρσενικόν");
    assert_eq!(Gender::Feminine.to_string(), "θηλυκόν");
    assert_eq!(Gender::Neuter.to_string(), "οὐδέτερον");
}

#[test]
fn test_enum_variants() {
    // Verify PartOfSpeech variants
    let _ = PartOfSpeech::Verb;
    let _ = PartOfSpeech::Adjective;
    let _ = PartOfSpeech::Pronoun;
    let _ = PartOfSpeech::Article;
    let _ = PartOfSpeech::Particle;
    let _ = PartOfSpeech::Numeral;
    let _ = PartOfSpeech::Preposition;
    let _ = PartOfSpeech::Conjunction;
    let _ = PartOfSpeech::Adverb;
    let _ = PartOfSpeech::Unknown;

    // Verify Person variants
    let _ = Person::First;
    let _ = Person::Second;
    let _ = Person::Third;

    // Verify Tense variants
    let _ = Tense::Present;
    let _ = Tense::Imperfect;
    let _ = Tense::Future;
    let _ = Tense::Aorist;
    let _ = Tense::Perfect;
    let _ = Tense::Pluperfect;

    // Verify Mood variants
    let _ = Mood::Indicative;
    let _ = Mood::Imperative;
    let _ = Mood::Subjunctive;
    let _ = Mood::Optative;
    let _ = Mood::Infinitive;
    let _ = Mood::Participle;

    // Verify Voice variants
    let _ = Voice::Active;
    let _ = Voice::Middle;
    let _ = Voice::Passive;
}
