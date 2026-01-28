//! Morphological analysis for Ancient Greek
//!
//! This module is the heart of ΓΛΩΣΣΑ. It analyzes Greek words to extract:
//! - Case (nominative, genitive, dative, accusative, vocative)
//! - Number (singular, plural)
//! - Gender (masculine, feminine, neuter)
//! - For verbs: person, tense, mood, voice
//!
//! ## Ambiguity Handling
//!
//! Greek morphology is inherently ambiguous. For example:
//! - `-α` can be nominative singular (1st decl.) or accusative plural (2nd decl. neuter)
//! - `-ης` can be nominative singular (1st decl. masc.) or genitive singular (1st decl. fem.)
//!
//! This module supports two analysis modes:
//! - `analyze()` - returns the most likely single analysis
//! - `analyze_all()` - returns ALL possible analyses with confidence scores
//!
//! Disambiguation uses syntactic context (article agreement, verb agreement) in the
//! semantic analysis phase.

mod case;
mod conjugation;
mod declension;
pub mod lexicon;
pub mod participle;

pub use case::*;
pub use conjugation::*;
pub use declension::*;
pub use lexicon::*;
pub use participle::*;

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
    /// Confidence score (0.0 - 1.0). Higher = more likely.
    /// Lexicon entries get 1.0, pattern matches get lower scores.
    pub confidence: f32,
}

impl MorphAnalysis {
    /// Create a new analysis with default confidence
    pub fn new(lemma: String, pos: PartOfSpeech) -> Self {
        MorphAnalysis {
            lemma,
            part_of_speech: pos,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 0.5,
        }
    }

    /// Set confidence score
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
}

/// Part of speech
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Pronoun,
    Article,
    Particle,
    Numeral,
    Preposition,
    Conjunction,
    Adverb,
    Unknown,
}

/// Analyze a Greek word and return the most likely morphological analysis
pub fn analyze(word: &str) -> MorphAnalysis {
    let analyses = analyze_all(word);
    analyses.into_iter().next().unwrap_or_else(|| {
        let normalized = normalize_greek(word);
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
            confidence: 0.0,
        }
    })
}

/// Analyze a Greek word and return ALL possible morphological analyses
///
/// Returns analyses sorted by confidence (highest first).
/// Use this when you need to resolve ambiguity using syntactic context.
///
/// # Example
/// ```
/// use glossa::morphology::analyze_all;
///
/// let analyses = analyze_all("θαλασσα");
/// // Could be:
/// // - Nominative singular feminine (1st decl.) - "the sea" as subject
/// // - Vocative singular feminine - "O sea!"
/// assert!(!analyses.is_empty());
/// ```
pub fn analyze_all(word: &str) -> Vec<MorphAnalysis> {
    let normalized = normalize_greek(word);
    let mut analyses = Vec::new();

    // First check the lexicon - these get highest confidence
    if let Some(entry) = lexicon::lookup(&normalized) {
        let mut analysis = entry.to_analysis();
        analysis.confidence = 1.0; // Lexicon entries are definitive
        analyses.push(analysis);
    }

    // Get all possible noun analyses
    let noun_analyses = declension::analyze_noun_all(&normalized);
    analyses.extend(noun_analyses);

    // Get all possible verb analyses
    let verb_analyses = conjugation::analyze_verb_all(&normalized);
    analyses.extend(verb_analyses);

    // Sort by confidence (highest first)
    analyses.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

    // If we found nothing, check if it's a single Greek letter (mathematical variable)
    if analyses.is_empty() || analyses.iter().all(|a| a.confidence < 0.5) {
        // Single Greek letters are treated as nominative nouns (variable names)
        // This follows mathematical convention where α, β, γ, ξ, etc. are variables
        if is_single_greek_letter(&normalized) {
            analyses.insert(
                0,
                MorphAnalysis {
                    lemma: normalized.clone(),
                    part_of_speech: PartOfSpeech::Noun,
                    case: Some(Case::Nominative),
                    number: Some(Number::Singular),
                    gender: None, // Unknown gender for variables
                    person: None,
                    tense: None,
                    mood: None,
                    voice: None,
                    confidence: 0.9, // High confidence for variable names
                },
            );
        }
    }

    // If still nothing, return unknown
    if analyses.is_empty() {
        analyses.push(MorphAnalysis {
            lemma: normalized,
            part_of_speech: PartOfSpeech::Unknown,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 0.0,
        });
    }

    analyses
}

/// Check if the word is a single Greek letter (used as mathematical variable)
fn is_single_greek_letter(word: &str) -> bool {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() != 1 {
        return false;
    }

    let c = chars[0];
    // Greek lowercase letters range: α (U+03B1) to ω (U+03C9)
    // Greek uppercase letters range: Α (U+0391) to Ω (U+03A9)
    // Also include ς (final sigma, U+03C2)
    ('\u{0391}'..='\u{03A9}').contains(&c) ||   // Uppercase
    ('\u{03B1}'..='\u{03C9}').contains(&c) ||   // Lowercase
    c == '\u{03C2}' // Final sigma
}

/// Check if two analyses are compatible (could refer to the same word in context)
pub fn analyses_compatible(a: &MorphAnalysis, b: &MorphAnalysis) -> bool {
    // Check case agreement if both have cases
    if let (Some(case_a), Some(case_b)) = (a.case, b.case) {
        if case_a != case_b {
            return false;
        }
    }

    // Check number agreement if both have numbers
    if let (Some(num_a), Some(num_b)) = (a.number, b.number) {
        if num_a != num_b {
            return false;
        }
    }

    // Check gender agreement if both have genders
    if let (Some(gen_a), Some(gen_b)) = (a.gender, b.gender) {
        if gen_a != gen_b {
            return false;
        }
    }

    true
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
