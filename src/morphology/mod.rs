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

pub mod conjugation;
pub mod declension;
pub mod disambiguation;
pub mod lexicon;
pub mod matcher;
pub mod models;
pub mod participle;

pub use conjugation::*;
pub use declension::*;
pub use disambiguation::*;
pub use lexicon::*;
pub use models::*;
pub use participle::*;

use std::borrow::Cow;

use crate::text::normalize_greek;

/// Safely compare two f32 confidence scores, treating NaN as Equal.
/// This prevents panics during sorting if a confidence calculation goes wrong.
#[inline]
pub(crate) fn compare_confidence(a: f32, b: f32) -> std::cmp::Ordering {
    // We usually sort descending, so caller should do `compare_confidence(b, a)`
    // or we can just provide the basic ascending comparison and let caller invert.
    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
}

/// Analyze a Greek word and return the most likely morphological analysis
pub fn analyze(word: &str) -> MorphAnalysis {
    let normalized = normalize_greek(word);

    // Optimization: Lexicon entries are definitive (confidence 1.0).
    // If found, we don't need to do expensive morphological analysis.
    // This avoids vector allocation and pattern matching for common words.
    if let Some(entry) = lexicon::lookup(&normalized) {
        let mut analysis = entry.to_analysis();
        analysis.confidence = 1.0;
        return analysis;
    }

    // analyze_all is guaranteed to return at least one analysis (Unknown if nothing else)
    analyze_all_from_normalized(&normalized)
        .into_iter()
        .next()
        .unwrap()
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
    analyze_all_from_normalized(&normalized)
}

/// Helper to analyze using an already-normalized string
fn analyze_all_from_normalized(normalized: &str) -> Vec<MorphAnalysis> {
    // Pre-allocate capacity to avoid reallocations
    // Lexicon (1) + Noun variants (~2-3) + Verb variants (~2-3)
    let mut analyses = Vec::with_capacity(8);

    // First check the lexicon - these get highest confidence
    if let Some(entry) = lexicon::lookup(normalized) {
        let mut analysis = entry.to_analysis();
        analysis.confidence = 1.0; // Lexicon entries are definitive
        analyses.push(analysis);
    }

    // Get all possible noun analyses (zero allocation)
    declension::analyze_noun_all_into(normalized, &mut analyses);

    // Get all possible verb analyses (zero allocation)
    conjugation::analyze_verb_all_into(normalized, &mut analyses);

    // Sort by confidence (highest first)
    analyses.sort_by(|a, b| compare_confidence(b.confidence, a.confidence));

    // If we found nothing, check if it's a single Greek letter (mathematical variable)
    if analyses.is_empty() || analyses.iter().all(|a| a.confidence < 0.5) {
        // Single Greek letters are treated as nominative nouns (variable names)
        // This follows mathematical convention where α, β, γ, ξ, etc. are variables
        if is_single_greek_letter(normalized) {
            analyses.insert(
                0,
                MorphAnalysis {
                    lemma: Cow::Owned(normalized.to_string()),
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
            lemma: Cow::Owned(normalized.to_string()),
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
///
/// Single letters like `α`, `β`, `π` are often used as variable names in ΓΛΩΣΣΑ.
/// This function identifies them so the fallback logic can treat them as
/// nominative nouns even if they aren't in the lexicon.
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_sort_safety_with_nan() {
        let mut analyses = [
            MorphAnalysis::new("test1".to_string(), PartOfSpeech::Noun).with_confidence(1.0),
            MorphAnalysis::new("test2".to_string(), PartOfSpeech::Noun).with_confidence(f32::NAN),
        ];

        // This uses the safe logic and should NOT panic
        analyses.sort_by(|a, b| compare_confidence(b.confidence, a.confidence));

        // Verify ordering is preserved for valid items (or at least no panic)
        // NaN comparisons are undefined, but we just ensure it doesn't crash
        assert_eq!(analyses.len(), 2);
    }

    #[test]
    fn test_unknown_word_fallback() {
        // "gibberish" should trigger the unknown fallback path
        let analysis = analyze("gibberish");
        assert_eq!(analysis.part_of_speech, PartOfSpeech::Unknown);
        assert_eq!(analysis.lemma, "gibberish"); // Should be Cow::Owned
        assert!(matches!(analysis.lemma, Cow::Owned(_)));
        assert_eq!(analysis.confidence, 0.0);
    }

    #[test]
    fn test_single_greek_letter_fallback() {
        // "α" should trigger the single greek letter fallback (mathematical variable)
        let analysis = analyze("α");
        assert_eq!(analysis.part_of_speech, PartOfSpeech::Noun);
        assert_eq!(analysis.case, Some(Case::Nominative));
        assert_eq!(analysis.lemma, "α"); // Should be Cow::Owned from clone
        // Note: lexicon entries might also cover some letters, so we check if it works generally
        // But specifically for a letter NOT in lexicon (if any), this fallback is key.
        // "ξ" (xi) is likely not in lexicon as a word
        let analysis_xi = analyze("ξ");
        assert_eq!(analysis_xi.part_of_speech, PartOfSpeech::Noun);
        assert_eq!(analysis_xi.lemma, "ξ");
    }

    #[test]
    fn test_ambiguous_word_analysis() {
        // "λόγον" is accusative singular, but let's check a word with multiple potential analyses
        // "α" is a good candidate: variable (noun), letter, etc.
        // Or "νέα": nominative singular feminine OR nominative plural neuter (if it was an adjective)

        // Let's use "λόγον" and ensure we get analyses
        let analyses = analyze_all("λόγον");
        assert!(!analyses.is_empty());

        // Ensure sorted by confidence
        for i in 0..analyses.len() - 1 {
            assert!(analyses[i].confidence >= analyses[i + 1].confidence);
        }
    }

    #[test]
    fn test_analyze_all_coverage_forms() {
        // Test various forms to ensure analyze_all (and underlying *_all functions) are covered

        // Neuter plural (hits SECOND_DECLENSION_NEUT in analyze_noun_all?)
        let analyses = analyze_all("δωρα");
        assert!(
            analyses
                .iter()
                .any(|a| a.lemma == "δωρον" && a.gender == Some(Gender::Neuter))
        );

        // Aorist Infinitive
        let analyses = analyze_all("λυσαι");
        assert!(analyses.iter().any(|a| a.mood == Some(Mood::Infinitive)));

        // Aorist Passive Optative
        let analyses = analyze_all("λυθειη");
        assert!(
            analyses
                .iter()
                .any(|a| a.mood == Some(Mood::Optative) && a.voice == Some(Voice::Passive))
        );

        // Aorist Indicative (with augment)
        let analyses = analyze_all("ελυσα");
        assert!(
            analyses
                .iter()
                .any(|a| a.tense == Some(Tense::Aorist) && a.lemma == "λυω")
        );
    }
}
