//! Noun declension tables and analysis
//!
//! Implements the three main declensions for ΓΛΩΣΣΑ:
//! - First declension: -η/-α feminine nouns
//! - Second declension: -ος/-ον masculine/neuter nouns
//! - Third declension: -μα neuter nouns (and others)

use std::borrow::Cow;

use super::{Case, Gender, MorphAnalysis, Number, PartOfSpeech};
use crate::morphology::matcher::match_suffix;

/// Declension pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Declension {
    First,  // -η/-α (feminine)
    Second, // -ος/-ον (masculine/neuter)
    Third,  // consonant stems, -μα
}

/// Second declension endings (masculine -ος type)
/// The most common pattern: λόγος, χρήστος, etc.
/// NOTE: Must be sorted by length descending!
const SECOND_DECLENSION_MASC: &[(&str, Case, Number)] = &[
    // Length 3
    ("οις", Case::Dative, Number::Plural),
    ("ους", Case::Accusative, Number::Plural),
    // Length 2
    ("ος", Case::Nominative, Number::Singular),
    ("ου", Case::Genitive, Number::Singular),
    ("ον", Case::Accusative, Number::Singular),
    ("οι", Case::Nominative, Number::Plural),
    ("ων", Case::Genitive, Number::Plural),
    // Length 1
    ("ω", Case::Dative, Number::Singular), // ῳ normalized to ω
    ("ε", Case::Vocative, Number::Singular),
];

/// Second declension endings (neuter -ον type)
/// Pattern: δῶρον, ἔργον, etc.
/// NOTE: Must be sorted by length descending!
const SECOND_DECLENSION_NEUT: &[(&str, Case, Number)] = &[
    // Length 3
    ("οις", Case::Dative, Number::Plural),
    // Length 2
    ("ον", Case::Nominative, Number::Singular),
    ("ου", Case::Genitive, Number::Singular),
    ("ον", Case::Accusative, Number::Singular),
    ("ον", Case::Vocative, Number::Singular),
    ("ων", Case::Genitive, Number::Plural),
    // Length 1
    ("ω", Case::Dative, Number::Singular),
    ("α", Case::Nominative, Number::Plural),
    ("α", Case::Accusative, Number::Plural),
];

/// First declension endings (-η type)
/// Pattern: τιμή, ψυχή, etc.
/// NOTE: Must be sorted by length descending!
const FIRST_DECLENSION_ETA: &[(&str, Case, Number)] = &[
    // Length 3
    ("αις", Case::Dative, Number::Plural),
    // Length 2
    ("ης", Case::Genitive, Number::Singular),
    ("ην", Case::Accusative, Number::Singular),
    ("αι", Case::Nominative, Number::Plural),
    ("ων", Case::Genitive, Number::Plural),
    ("ας", Case::Accusative, Number::Plural),
    // Length 1
    ("η", Case::Nominative, Number::Singular),
    ("η", Case::Dative, Number::Singular), // ῃ normalized
    ("η", Case::Vocative, Number::Singular),
];

/// First declension endings (-α type, pure alpha)
/// Pattern: χώρα, θάλαττα, etc.
/// NOTE: Must be sorted by length descending!
const FIRST_DECLENSION_ALPHA: &[(&str, Case, Number)] = &[
    // Length 3
    ("αις", Case::Dative, Number::Plural),
    // Length 2
    ("ας", Case::Genitive, Number::Singular),
    ("αν", Case::Accusative, Number::Singular),
    ("αι", Case::Nominative, Number::Plural),
    ("ων", Case::Genitive, Number::Plural),
    ("ας", Case::Accusative, Number::Plural),
    // Length 1
    ("α", Case::Nominative, Number::Singular),
    ("α", Case::Dative, Number::Singular), // ᾳ normalized
    ("α", Case::Vocative, Number::Singular),
];

/// Third declension endings (-μα type)
/// Pattern: ὄνομα, πρᾶγμα, σῶμα, etc.
/// NOTE: Must be sorted by length descending!
const THIRD_DECLENSION_MA: &[(&str, Case, Number)] = &[
    // Length 5
    ("ματος", Case::Genitive, Number::Singular),
    ("ματων", Case::Genitive, Number::Plural),
    // Length 4
    ("ματι", Case::Dative, Number::Singular),
    ("ματα", Case::Nominative, Number::Plural),
    ("μασι", Case::Dative, Number::Plural), // μασι(ν)
    ("ματα", Case::Accusative, Number::Plural),
    // Length 2
    ("μα", Case::Nominative, Number::Singular),
    ("μα", Case::Accusative, Number::Singular),
    ("μα", Case::Vocative, Number::Singular),
];

/// Declension configuration pattern
struct DeclensionPattern {
    endings: &'static [(&'static str, Case, Number)],
    gender: Gender,
    nom_ending: &'static str,
    base_confidence: f32,
}

/// Ordered list of declension patterns for analysis.
/// Order matters for `analyze_noun` (first match wins).
const DECLENSION_PATTERNS: &[DeclensionPattern] = &[
    DeclensionPattern {
        endings: THIRD_DECLENSION_MA,
        gender: Gender::Neuter,
        nom_ending: "μα",
        base_confidence: 0.9, // -μα is distinctive
    },
    DeclensionPattern {
        endings: SECOND_DECLENSION_MASC,
        gender: Gender::Masculine,
        nom_ending: "ος",
        base_confidence: 0.8,
    },
    DeclensionPattern {
        endings: SECOND_DECLENSION_NEUT,
        gender: Gender::Neuter,
        nom_ending: "ον",
        base_confidence: 0.75, // Note: was 0.75 in analyze_noun, 0.7 in analyze_noun_all. Using 0.75 to preserve analyze_noun behavior.
    },
    DeclensionPattern {
        endings: FIRST_DECLENSION_ETA,
        gender: Gender::Feminine,
        nom_ending: "η",
        base_confidence: 0.8,
    },
    DeclensionPattern {
        endings: FIRST_DECLENSION_ALPHA,
        gender: Gender::Feminine,
        nom_ending: "α",
        base_confidence: 0.75, // Note: was 0.75 in analyze_noun, 0.7 in analyze_noun_all.
    },
];

/// Try to analyze a word as a noun by matching declension endings
pub fn analyze_noun(word: &str) -> Option<MorphAnalysis> {
    let mut analyses = analyze_noun_all(word);

    // Sort by confidence (highest first)
    // analyze_noun_all sorts by (case, number, gender, lemma) but NOT confidence.
    // So we sort by confidence here. Since sort_by is stable, it preserves the
    // order for equal confidence (which prefers singular/simpler forms).
    analyses.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    analyses.into_iter().next()
}

/// Match a word against ALL possible endings, calling callback for each match
fn match_endings_all<F>(word: &str, endings: &[(&str, Case, Number)], mut callback: F)
where
    F: FnMut(&str, Case, Number),
{
    match_suffix(word, endings, |e| e.0, |stem, pattern| {
        callback(stem, pattern.1, pattern.2);
        true
    });
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
    analyze_noun_all_into(word, &mut analyses);
    analyses
}

/// Analyze a word as a noun, pushing results into an existing vector
///
/// Zero-allocation version of `analyze_noun_all`.
pub fn analyze_noun_all_into(word: &str, analyses: &mut Vec<MorphAnalysis>) {
    let start_len = analyses.len();

    for decl in DECLENSION_PATTERNS {
        match_endings_all(word, decl.endings, |stem, case, number| {
            // Calculate confidence based on ending length and distinctiveness
            let ending_len = word.len() - stem.len();
            let length_bonus = (ending_len as f32 - 1.0) * 0.05; // Longer = better

            // Adjust base confidence if needed to match original behavior
            // Original analyze_noun_all used 0.7 for neuter/alpha, but analyze_noun uses 0.75.
            // We standardized on 0.75 in the struct.
            // Increased cap from 0.95 to 0.99 to allow highly distinctive endings (like -ματος)
            // to outscore less distinctive ones (like -ος) even when both have high base confidence.
            let confidence = (decl.base_confidence + length_bonus).min(0.99);

            // Optimization: Avoid format! for canonical forms
            let lemma = if word.len() == stem.len() + decl.nom_ending.len()
                && word.ends_with(decl.nom_ending)
            {
                Cow::Owned(word.to_string())
            } else {
                Cow::Owned(format!("{}{}", stem, decl.nom_ending))
            };

            analyses.push(MorphAnalysis {
                lemma,
                part_of_speech: PartOfSpeech::Noun,
                case: Some(case),
                number: Some(number),
                gender: Some(decl.gender),
                person: Some(crate::morphology::Person::Third),
                tense: None,
                mood: None,
                voice: None,
                confidence,
            });
        });
    }

    // Sort the newly added analyses (the tail)
    analyses[start_len..].sort_by(|a, b| {
        let key_a = (a.case, a.number, a.gender, &a.lemma);
        let key_b = (b.case, b.number, b.gender, &b.lemma);
        key_a.cmp(&key_b)
    });

    // Deduplicate identical analyses in the tail
    let len = analyses.len();
    if len > start_len + 1 {
        let mut w = start_len + 1;
        for r in start_len + 1..len {
            let matches = {
                let a = &analyses[w - 1];
                let b = &analyses[r];
                a.case == b.case
                    && a.number == b.number
                    && a.gender == b.gender
                    && a.lemma == b.lemma
            };

            if !matches {
                if r != w {
                    analyses.swap(r, w);
                }
                w += 1;
            }
        }
        analyses.truncate(w);
    }
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
        (Declension::First, _) => {
            // Check if stem ends in epsilon, iota, or rho (Alpha pure rule)
            // Note: Input stem should be normalized (lowercase, monotonic)
            // Long Alpha type: -α, -ας, -ᾳ, -αν
            if stem.ends_with('ε') || stem.ends_with('ι') || stem.ends_with('ρ') {
                FIRST_DECLENSION_ALPHA
            } else {
                // Eta type: -η, -ης, -ῃ, -ην (or mixed alpha)
                FIRST_DECLENSION_ETA
            }
        }
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
    fn test_constants_sorted() {
        // Enforce that ending constants are sorted by length descending
        let constant_lists = vec![
            ("SECOND_DECLENSION_MASC", SECOND_DECLENSION_MASC),
            ("SECOND_DECLENSION_NEUT", SECOND_DECLENSION_NEUT),
            ("FIRST_DECLENSION_ETA", FIRST_DECLENSION_ETA),
            ("FIRST_DECLENSION_ALPHA", FIRST_DECLENSION_ALPHA),
            ("THIRD_DECLENSION_MA", THIRD_DECLENSION_MA),
        ];

        for (name, list) in constant_lists {
            for (i, window) in list.windows(2).enumerate() {
                let current = window[0].0;
                let next = window[1].0;
                let current_len = current.len();
                let next_len = next.len();
                assert!(
                    current_len >= next_len,
                    "{} is not sorted by length descending! Element at {} ('{}', len {}) is shorter than element at {} ('{}', len {})",
                    name,
                    i,
                    current,
                    current_len,
                    i + 1,
                    next,
                    next_len
                );
            }
        }
    }

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
    fn test_second_declension_neuter_plural() {
        // "δωρα" (gifts) - Neuter Plural (ends in -α)
        // This relies on SECOND_DECLENSION_NEUT fallback.
        // Note: "δωρα" is morphologically ambiguous with First Declension Alpha (Singular).
        // analyze_noun might prefer Singular due to tie-breaking rules.
        // So we use analyze_noun_all to verify the Neuter Plural analysis exists.
        let analyses = analyze_noun_all("δωρα");

        let found = analyses.iter().find(|a| {
            a.part_of_speech == PartOfSpeech::Noun
                && a.lemma == "δωρον"
                && a.gender == Some(Gender::Neuter)
                && a.case == Some(Case::Nominative) // or Accusative
                && a.number == Some(Number::Plural)
        });

        assert!(
            found.is_some(),
            "Should find Second Declension Neuter Plural analysis for δωρα"
        );
    }

    #[test]
    fn test_analyze_noun_disambiguation_alpha() {
        // "χώρα" (country) ends in -α.
        // Should be identified as First Declension Feminine Singular Nominative
        // over Second Declension Neuter Plural Nominative, because Singular is preferred in ties
        // or confidence rules should handle it.
        let analysis = analyze_noun("χωρα").expect("Should analyze χωρα");

        assert_eq!(analysis.gender, Some(Gender::Feminine));
        assert_eq!(analysis.number, Some(Number::Singular));
        assert_eq!(analysis.lemma, "χωρα");
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
        // Updated to assert the CORRECT behavior after fix
        // "χωρ" (stem of χώρα) + First Declension + Genitive Singular
        // Expectation for Alpha type: "χωρας" (pure alpha)

        let result = decline(
            "χωρ",
            Declension::First,
            Gender::Feminine,
            Case::Genitive,
            Number::Singular,
        );

        assert_eq!(
            result, "χωρας",
            "Should return alpha ending for stem ending in rho"
        );
    }

    #[test]
    fn test_decline_table() {
        // Table-driven test for decline() covering multiple declensions and cases
        struct TestCase {
            stem: &'static str,
            declension: Declension,
            gender: Gender,
            case: Case,
            number: Number,
            expected: &'static str,
            description: &'static str,
        }

        let cases = vec![
            // First Declension - Eta type (stem ending in consonant other than rho)
            TestCase {
                stem: "τιμ",
                declension: Declension::First,
                gender: Gender::Feminine,
                case: Case::Nominative,
                number: Number::Singular,
                expected: "τιμη",
                description: "First Declension Eta Nominative Singular",
            },
            TestCase {
                stem: "τιμ",
                declension: Declension::First,
                gender: Gender::Feminine,
                case: Case::Genitive,
                number: Number::Singular,
                expected: "τιμης",
                description: "First Declension Eta Genitive Singular",
            },
            // First Declension - Alpha type (stem ending in rho)
            TestCase {
                stem: "χωρ",
                declension: Declension::First,
                gender: Gender::Feminine,
                case: Case::Nominative,
                number: Number::Singular,
                expected: "χωρα",
                description: "First Declension Alpha (rho) Nominative Singular",
            },
            TestCase {
                stem: "χωρ",
                declension: Declension::First,
                gender: Gender::Feminine,
                case: Case::Genitive,
                number: Number::Singular,
                expected: "χωρας",
                description: "First Declension Alpha (rho) Genitive Singular",
            },
            // First Declension - Alpha type (stem ending in iota)
            TestCase {
                stem: "οικι",
                declension: Declension::First,
                gender: Gender::Feminine,
                case: Case::Genitive,
                number: Number::Singular,
                expected: "οικιας",
                description: "First Declension Alpha (iota) Genitive Singular",
            },
            // First Declension - Alpha type (stem ending in epsilon)
            // e.g. γενεά -> γενε
            TestCase {
                stem: "γενε",
                declension: Declension::First,
                gender: Gender::Feminine,
                case: Case::Genitive,
                number: Number::Singular,
                expected: "γενεας",
                description: "First Declension Alpha (epsilon) Genitive Singular",
            },
            // Second Declension - Masculine
            TestCase {
                stem: "λογ",
                declension: Declension::Second,
                gender: Gender::Masculine,
                case: Case::Accusative,
                number: Number::Plural,
                expected: "λογους",
                description: "Second Declension Masculine Accusative Plural",
            },
            // Second Declension - Neuter
            TestCase {
                stem: "δωρ",
                declension: Declension::Second,
                gender: Gender::Neuter,
                case: Case::Nominative,
                number: Number::Plural,
                expected: "δωρα",
                description: "Second Declension Neuter Nominative Plural",
            },
            // Third Declension - Neuter (-μα type)
            TestCase {
                stem: "σω", // σῶμα -> σώματος
                declension: Declension::Third,
                gender: Gender::Neuter,
                case: Case::Genitive,
                number: Number::Singular,
                expected: "σωματος",
                description: "Third Declension Neuter Genitive Singular",
            },
        ];

        for test in cases {
            let result = decline(
                test.stem,
                test.declension,
                test.gender,
                test.case,
                test.number,
            );
            assert_eq!(
                result, test.expected,
                "Failed test: {}. Stem: {}, Decl: {:?}, Gender: {:?}, Case: {:?}, Num: {:?}",
                test.description, test.stem, test.declension, test.gender, test.case, test.number
            );
        }
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
