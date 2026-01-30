//! Noun declension tables and analysis
//!
//! Implements the three main declensions for ΓΛΩΣΣΑ:
//! - First declension: -η/-α feminine nouns
//! - Second declension: -ος/-ον masculine/neuter nouns
//! - Third declension: -μα neuter nouns (and others)

use super::{Case, Gender, MorphAnalysis, Number, PartOfSpeech};

/// Declension pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Declension {
    First,  // -η/-α (feminine)
    Second, // -ος/-ον (masculine/neuter)
    Third,  // consonant stems, -μα
}

/// Second declension endings (masculine -ος type)
/// The most common pattern: λόγος, χρήστος, etc.
const SECOND_DECLENSION_MASC: &[(&str, Case, Number)] = &[
    // Singular
    ("ος", Case::Nominative, Number::Singular),
    ("ου", Case::Genitive, Number::Singular),
    ("ω", Case::Dative, Number::Singular), // ῳ normalized to ω
    ("ον", Case::Accusative, Number::Singular),
    ("ε", Case::Vocative, Number::Singular),
    // Plural
    ("οι", Case::Nominative, Number::Plural),
    ("ων", Case::Genitive, Number::Plural),
    ("οις", Case::Dative, Number::Plural),
    ("ους", Case::Accusative, Number::Plural),
];

/// Second declension endings (neuter -ον type)
/// Pattern: δῶρον, ἔργον, etc.
const SECOND_DECLENSION_NEUT: &[(&str, Case, Number)] = &[
    // Singular
    ("ον", Case::Nominative, Number::Singular),
    ("ου", Case::Genitive, Number::Singular),
    ("ω", Case::Dative, Number::Singular),
    ("ον", Case::Accusative, Number::Singular),
    ("ον", Case::Vocative, Number::Singular),
    // Plural
    ("α", Case::Nominative, Number::Plural),
    ("ων", Case::Genitive, Number::Plural),
    ("οις", Case::Dative, Number::Plural),
    ("α", Case::Accusative, Number::Plural),
];

/// First declension endings (-η type)
/// Pattern: τιμή, ψυχή, etc.
const FIRST_DECLENSION_ETA: &[(&str, Case, Number)] = &[
    // Singular
    ("η", Case::Nominative, Number::Singular),
    ("ης", Case::Genitive, Number::Singular),
    ("η", Case::Dative, Number::Singular), // ῃ normalized
    ("ην", Case::Accusative, Number::Singular),
    ("η", Case::Vocative, Number::Singular),
    // Plural
    ("αι", Case::Nominative, Number::Plural),
    ("ων", Case::Genitive, Number::Plural),
    ("αις", Case::Dative, Number::Plural),
    ("ας", Case::Accusative, Number::Plural),
];

/// First declension endings (-α type, pure alpha)
/// Pattern: χώρα, θάλαττα, etc.
const FIRST_DECLENSION_ALPHA: &[(&str, Case, Number)] = &[
    // Singular
    ("α", Case::Nominative, Number::Singular),
    ("ας", Case::Genitive, Number::Singular),
    ("α", Case::Dative, Number::Singular), // ᾳ normalized
    ("αν", Case::Accusative, Number::Singular),
    ("α", Case::Vocative, Number::Singular),
    // Plural (same as eta type)
    ("αι", Case::Nominative, Number::Plural),
    ("ων", Case::Genitive, Number::Plural),
    ("αις", Case::Dative, Number::Plural),
    ("ας", Case::Accusative, Number::Plural),
];

/// Third declension endings (-μα type)
/// Pattern: ὄνομα, πρᾶγμα, σῶμα, etc.
const THIRD_DECLENSION_MA: &[(&str, Case, Number)] = &[
    // Singular
    ("μα", Case::Nominative, Number::Singular),
    ("ματος", Case::Genitive, Number::Singular),
    ("ματι", Case::Dative, Number::Singular),
    ("μα", Case::Accusative, Number::Singular),
    ("μα", Case::Vocative, Number::Singular),
    // Plural
    ("ματα", Case::Nominative, Number::Plural),
    ("ματων", Case::Genitive, Number::Plural),
    ("μασι", Case::Dative, Number::Plural), // μασι(ν)
    ("ματα", Case::Accusative, Number::Plural),
];

/// Try to analyze a word as a noun by matching declension endings
pub fn analyze_noun(word: &str) -> Option<MorphAnalysis> {
    // Try each declension pattern, longest ending first

    // Third declension -μα (check longer endings first)
    if let Some((stem, case, number)) = match_endings(word, THIRD_DECLENSION_MA) {
        return Some(MorphAnalysis {
            lemma: format!("{}μα", stem),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(case),
            number: Some(number),
            gender: Some(Gender::Neuter),
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 0.9,
        });
    }

    // Second declension masculine -ος
    if let Some((stem, case, number)) = match_endings(word, SECOND_DECLENSION_MASC) {
        return Some(MorphAnalysis {
            lemma: format!("{}ος", stem),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(case),
            number: Some(number),
            gender: Some(Gender::Masculine),
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 0.8,
        });
    }

    // Second declension neuter -ον
    if let Some((stem, case, number)) = match_endings(word, SECOND_DECLENSION_NEUT) {
        // Only if it doesn't match masculine (which has priority for -ον accusative)
        if !word.ends_with("ος") {
            return Some(MorphAnalysis {
                lemma: format!("{}ον", stem),
                part_of_speech: PartOfSpeech::Noun,
                case: Some(case),
                number: Some(number),
                gender: Some(Gender::Neuter),
                person: None,
                tense: None,
                mood: None,
                voice: None,
                confidence: 0.75,
            });
        }
    }

    // First declension -η
    if let Some((stem, case, number)) = match_endings(word, FIRST_DECLENSION_ETA) {
        return Some(MorphAnalysis {
            lemma: format!("{}η", stem),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(case),
            number: Some(number),
            gender: Some(Gender::Feminine),
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 0.8,
        });
    }

    // First declension -α
    if let Some((stem, case, number)) = match_endings(word, FIRST_DECLENSION_ALPHA) {
        return Some(MorphAnalysis {
            lemma: format!("{}α", stem),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(case),
            number: Some(number),
            gender: Some(Gender::Feminine),
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 0.75,
        });
    }

    None
}

/// Match a word against a list of endings, returning the stem and grammatical info
fn match_endings(word: &str, endings: &[(&str, Case, Number)]) -> Option<(String, Case, Number)> {
    // Sort by ending length (longest first) to match most specific
    let mut sorted_endings: Vec<_> = endings.iter().collect();
    sorted_endings.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (ending, case, number) in sorted_endings {
        if let Some(stem) = word.strip_suffix(ending)
            && !stem.is_empty()
        {
            return Some((stem.to_string(), *case, *number));
        }
    }
    None
}

/// Match a word against ALL possible endings, invoking a callback for each match
///
/// Optimization: Uses a callback instead of returning a Vec to avoid heap allocations.
fn match_endings_all<F>(word: &str, endings: &[(&str, Case, Number)], mut callback: F)
where
    F: FnMut(&str, Case, Number),
{
    for (ending, case, number) in endings {
        if let Some(stem) = word.strip_suffix(ending)
            && !stem.is_empty()
        {
            callback(stem, *case, *number);
        }
    }
}

/// Analyze a word as a noun, returning ALL possible analyses
///
/// This is used for ambiguity resolution. For example, "θαλασσα" could be:
/// - Nominative singular (the sea as subject)
/// - Vocative singular (O sea!)
///
/// The caller should use syntactic context to pick the right one.
pub fn analyze_noun_all(word: &str) -> Vec<MorphAnalysis> {
    let mut analyses = Vec::new();

    // Declension info for confidence scoring
    // Longer endings = higher confidence
    struct DeclInfo {
        endings: &'static [(&'static str, Case, Number)],
        gender: Gender,
        nom_ending: &'static str,
        base_confidence: f32,
    }

    let declensions = [
        DeclInfo {
            endings: THIRD_DECLENSION_MA,
            gender: Gender::Neuter,
            nom_ending: "μα",
            base_confidence: 0.9, // -μα is distinctive
        },
        DeclInfo {
            endings: SECOND_DECLENSION_MASC,
            gender: Gender::Masculine,
            nom_ending: "ος",
            base_confidence: 0.8,
        },
        DeclInfo {
            endings: SECOND_DECLENSION_NEUT,
            gender: Gender::Neuter,
            nom_ending: "ον",
            base_confidence: 0.7,
        },
        DeclInfo {
            endings: FIRST_DECLENSION_ETA,
            gender: Gender::Feminine,
            nom_ending: "η",
            base_confidence: 0.75,
        },
        DeclInfo {
            endings: FIRST_DECLENSION_ALPHA,
            gender: Gender::Feminine,
            nom_ending: "α",
            base_confidence: 0.7,
        },
    ];

    for decl in &declensions {
        match_endings_all(word, decl.endings, |stem, case, number| {
            // Calculate confidence based on ending length and distinctiveness
            let ending_len = word.len() - stem.len();
            let length_bonus = (ending_len as f32 - 1.0) * 0.05; // Longer = better
            let confidence = (decl.base_confidence + length_bonus).min(0.95);

            analyses.push(MorphAnalysis {
                lemma: format!("{}{}", stem, decl.nom_ending),
                part_of_speech: PartOfSpeech::Noun,
                case: Some(case),
                number: Some(number),
                gender: Some(decl.gender),
                person: None,
                tense: None,
                mood: None,
                voice: None,
                confidence,
            });
        });
    }

    // Deduplicate identical analyses (same case/number/gender)
    analyses.sort_by(|a, b| {
        let key_a = (a.case, a.number, a.gender, &a.lemma);
        let key_b = (b.case, b.number, b.gender, &b.lemma);
        key_a.cmp(&key_b)
    });
    analyses.dedup_by(|a, b| {
        a.case == b.case && a.number == b.number && a.gender == b.gender && a.lemma == b.lemma
    });

    analyses
}

/// Extract stem from a word given its nominative form and declension
pub fn get_stem(nominative: &str, declension: Declension) -> String {
    match declension {
        Declension::Second => {
            if let Some(stem) = nominative.strip_suffix("ος") {
                stem.to_string()
            } else if let Some(stem) = nominative.strip_suffix("ον") {
                stem.to_string()
            } else {
                nominative.to_string()
            }
        }
        Declension::First => {
            if let Some(stem) = nominative.strip_suffix("η") {
                stem.to_string()
            } else if let Some(stem) = nominative.strip_suffix("α") {
                stem.to_string()
            } else {
                nominative.to_string()
            }
        }
        Declension::Third => {
            if let Some(stem) = nominative.strip_suffix("μα") {
                stem.to_string()
            } else {
                nominative.to_string()
            }
        }
    }
}

/// Decline a noun to a specific case and number
pub fn decline(
    stem: &str,
    declension: Declension,
    gender: Gender,
    case: Case,
    number: Number,
) -> String {
    let endings = match (declension, gender) {
        (Declension::Second, Gender::Masculine) => SECOND_DECLENSION_MASC,
        (Declension::Second, Gender::Neuter) => SECOND_DECLENSION_NEUT,
        (Declension::First, _) => FIRST_DECLENSION_ETA,
        (Declension::Third, Gender::Neuter) => THIRD_DECLENSION_MA,
        _ => return stem.to_string(),
    };

    for (ending, c, n) in endings {
        if *c == case && *n == number {
            return format!("{}{}", stem, ending);
        }
    }

    stem.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_second_declension_nominative() {
        let analysis = analyze_noun("χρηστος").unwrap();
        assert_eq!(analysis.case, Some(Case::Nominative));
        assert_eq!(analysis.number, Some(Number::Singular));
        assert_eq!(analysis.gender, Some(Gender::Masculine));
    }

    #[test]
    fn test_second_declension_genitive() {
        let analysis = analyze_noun("χρηστου").unwrap();
        assert_eq!(analysis.case, Some(Case::Genitive));
        assert_eq!(analysis.number, Some(Number::Singular));
        assert_eq!(analysis.lemma, "χρηστος");
    }

    #[test]
    fn test_second_declension_dative() {
        let analysis = analyze_noun("χρηστω").unwrap();
        assert_eq!(analysis.case, Some(Case::Dative));
        assert_eq!(analysis.number, Some(Number::Singular));
    }

    #[test]
    fn test_second_declension_accusative() {
        let analysis = analyze_noun("χρηστον").unwrap();
        assert_eq!(analysis.case, Some(Case::Accusative));
        assert_eq!(analysis.number, Some(Number::Singular));
    }

    #[test]
    fn test_second_declension_vocative() {
        let analysis = analyze_noun("χρηστε").unwrap();
        assert_eq!(analysis.case, Some(Case::Vocative));
        assert_eq!(analysis.number, Some(Number::Singular));
    }

    #[test]
    fn test_second_declension_plural() {
        let analysis = analyze_noun("χρηστοι").unwrap();
        assert_eq!(analysis.case, Some(Case::Nominative));
        assert_eq!(analysis.number, Some(Number::Plural));

        let analysis = analyze_noun("χρηστων").unwrap();
        assert_eq!(analysis.case, Some(Case::Genitive));
        assert_eq!(analysis.number, Some(Number::Plural));
    }

    #[test]
    fn test_first_declension_eta() {
        let analysis = analyze_noun("λιστη").unwrap();
        assert_eq!(analysis.case, Some(Case::Nominative));
        assert_eq!(analysis.gender, Some(Gender::Feminine));

        let analysis = analyze_noun("λιστης").unwrap();
        assert_eq!(analysis.case, Some(Case::Genitive));
    }

    #[test]
    fn test_third_declension_ma() {
        let analysis = analyze_noun("ονομα").unwrap();
        assert_eq!(analysis.case, Some(Case::Nominative));
        assert_eq!(analysis.gender, Some(Gender::Neuter));

        let analysis = analyze_noun("ονοματος").unwrap();
        assert_eq!(analysis.case, Some(Case::Genitive));
        // The lemma is correctly reconstructed as "ονομα" (stem "ονο" + "μα")
        assert_eq!(analysis.lemma, "ονομα");
    }

    #[test]
    fn test_decline_second_masculine() {
        assert_eq!(
            decline(
                "χρηστ",
                Declension::Second,
                Gender::Masculine,
                Case::Nominative,
                Number::Singular
            ),
            "χρηστος"
        );
        assert_eq!(
            decline(
                "χρηστ",
                Declension::Second,
                Gender::Masculine,
                Case::Genitive,
                Number::Singular
            ),
            "χρηστου"
        );
        assert_eq!(
            decline(
                "χρηστ",
                Declension::Second,
                Gender::Masculine,
                Case::Dative,
                Number::Singular
            ),
            "χρηστω"
        );
    }

    #[test]
    fn test_get_stem() {
        assert_eq!(get_stem("λογος", Declension::Second), "λογ");
        assert_eq!(get_stem("τιμη", Declension::First), "τιμ");
        assert_eq!(get_stem("ονομα", Declension::Third), "ονο");
    }

    #[test]
    fn test_analyze_noun_all_ambiguity() {
        // "σωμα" (body) is Third Declension neuter
        // It has the ending -μα which matches Nominative, Accusative, and Vocative Singular
        // But -α ending also matches First Declension feminine
        let analyses = analyze_noun_all("σωμα");

        // Should have at least 3 analyses
        assert!(
            analyses.len() >= 6,
            "Expected at least 6 analyses for ambiguity, got {}",
            analyses.len()
        );

        // Check that we found the correct Third Declension Neuter analysis
        // This should have higher confidence because "μα" (len 2) matches better than "α" (len 1)
        let found_neuter = analyses.iter().any(|a| {
            a.gender == Some(Gender::Neuter)
                && a.number == Some(Number::Singular)
                && a.lemma == "σωμα"
        });
        assert!(found_neuter, "Should find Third Declension Neuter analysis");

        // We might also find First Declension Feminine (lemma "σωμα" from stem "σωμ" + "α")
        let found_feminine = analyses.iter().any(|a| {
            a.gender == Some(Gender::Feminine)
                && a.number == Some(Number::Singular)
                && a.lemma == "σωμα"
        });
        assert!(
            found_feminine,
            "Should find First Declension Feminine singular analysis (ambiguity)"
        );
    }

    #[test]
    fn test_first_declension_alpha() {
        // "χώρα" (country) - First Declension Alpha type
        // Using analyze_noun_all because analyze_noun (singular) defaults to
        // Second Declension Plural (neuter) for -α ending (e.g. δῶρα from δῶρον).
        let analyses = analyze_noun_all("χωρα");

        let found = analyses.iter().find(|a| {
            a.part_of_speech == PartOfSpeech::Noun
                && a.lemma == "χωρα"
                && a.gender == Some(Gender::Feminine)
                && a.case == Some(Case::Nominative)
        });

        assert!(
            found.is_some(),
            "Should find First Declension Alpha analysis for χωρα"
        );
    }

    #[test]
    fn test_decline_alpha_mismatch() {
        // This test documents a known limitation in `decline`
        // It uses FIRST_DECLENSION_ETA for all First Declension nouns

        // "χωρ" (stem of χώρα) + First Declension + Genitive Singular
        // Expectation for Alpha type: "χωρας"
        // Actual behavior (Eta type): "χωρης"

        let result = decline(
            "χωρ",
            Declension::First,
            Gender::Feminine,
            Case::Genitive,
            Number::Singular,
        );

        // Assert the current behavior (Eta ending)
        assert_eq!(result, "χωρης");

        // If we fix the bug, this test should be updated to expect "χωρας"
    }

    #[test]
    fn test_decline_fallthrough() {
        // Test that decline returns stem as-is if no ending matches
        // E.g. asking for a case that doesn't exist in the table (unlikely with Case enum)
        // or asking for a declension/gender combo that isn't handled.

        // Declension::Third + Gender::Masculine is NOT handled in decline()
        let stem = "αγνωστ";
        let result = decline(
            stem,
            Declension::Third,
            Gender::Masculine,
            Case::Nominative,
            Number::Singular,
        );

        assert_eq!(result, stem);
    }
}
