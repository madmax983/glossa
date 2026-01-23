//! Morphological analysis for Ancient Greek
//!
//! This module is the heart of ΓΛΩΣΣΑ. It analyzes Greek words to extract:
//! - Case (nominative, genitive, dative, accusative, vocative)
//! - Number (singular, plural)
//! - Gender (masculine, feminine, neuter)
//! - For verbs: person, tense, mood, voice

mod case;
mod declension;
mod conjugation;
pub mod lexicon;

pub use case::*;
pub use declension::*;
pub use conjugation::*;
pub use lexicon::*;

use crate::grammar::normalize_greek;

/// Result of morphological analysis
#[derive(Debug, Clone, PartialEq)]
pub struct MorphAnalysis {
    pub lemma: String,
    pub part_of_speech: PartOfSpeech,
    pub case: Option<Case>,
    pub number: Option<Number>,
    pub gender: Option<Gender>,
    pub person: Option<Person>,
    pub tense: Option<Tense>,
    pub mood: Option<Mood>,
    pub voice: Option<Voice>,
}

/// Part of speech
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Pronoun,
    Particle,
    Numeral,
    Unknown,
}

/// Analyze a Greek word and return its morphological features
pub fn analyze(word: &str) -> MorphAnalysis {
    let normalized = normalize_greek(word);

    // First check the lexicon for known words
    if let Some(entry) = lexicon::lookup(&normalized) {
        return entry.to_analysis();
    }

    // Try to analyze as a noun by declension patterns
    if let Some(analysis) = declension::analyze_noun(&normalized) {
        return analysis;
    }

    // Try to analyze as a verb by conjugation patterns
    if let Some(analysis) = conjugation::analyze_verb(&normalized) {
        return analysis;
    }

    // Unknown word
    MorphAnalysis {
        lemma: normalized,
        part_of_speech: PartOfSpeech::Unknown,
        case: None,
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_nominative() {
        let analysis = analyze("χρήστος");
        assert_eq!(analysis.case, Some(Case::Nominative));
        assert_eq!(analysis.number, Some(Number::Singular));
    }

    #[test]
    fn test_analyze_genitive() {
        let analysis = analyze("χρήστου");
        assert_eq!(analysis.case, Some(Case::Genitive));
        assert_eq!(analysis.number, Some(Number::Singular));
    }

    #[test]
    fn test_analyze_dative() {
        let analysis = analyze("χρήστῳ");
        assert_eq!(analysis.case, Some(Case::Dative));
        assert_eq!(analysis.number, Some(Number::Singular));
    }

    #[test]
    fn test_analyze_accusative() {
        let analysis = analyze("χρήστον");
        assert_eq!(analysis.case, Some(Case::Accusative));
        assert_eq!(analysis.number, Some(Number::Singular));
    }

    #[test]
    fn test_analyze_verb_imperative() {
        let analysis = analyze("λέγε");
        assert_eq!(analysis.part_of_speech, PartOfSpeech::Verb);
        assert_eq!(analysis.mood, Some(Mood::Imperative));
    }

    #[test]
    fn test_analyze_verb_present() {
        let analysis = analyze("γράφει");
        assert_eq!(analysis.part_of_speech, PartOfSpeech::Verb);
        assert_eq!(analysis.tense, Some(Tense::Present));
        assert_eq!(analysis.person, Some(Person::Third));
    }

    #[test]
    fn test_analyze_lexicon_lookup() {
        let analysis = analyze("ἔστω");
        assert_eq!(analysis.part_of_speech, PartOfSpeech::Verb);
        assert_eq!(analysis.mood, Some(Mood::Imperative));
    }
}
