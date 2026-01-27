//! Participle analysis for Ancient Greek
//!
//! Greek participles are verbal adjectives that combine verb properties (tense, voice)
//! with adjective properties (case, gender, number). In ΓΛΩΣΣΑ, participles serve as
//! the primary mechanism for creating lambdas and closures.
//!
//! ## Participle-Lambda Mapping
//!
//! | Participle | Example | Lambda Type |
//! |------------|---------|-------------|
//! | Present Active | γράφων "writing" | Streaming closure `\|x\| body` |
//! | Present Middle | διπλασιαζόμενον "doubling itself" | Method closure `\|x\| x.method()` |
//! | Aorist Active | γράψας "having written" | One-shot closure `move \|x\| body` |
//! | Perfect Passive | γεγραμμένος "having been written" | Memoized closure |
//!
//! ## Examples
//!
//! ```ignore
//! use glossa::morphology::participle::analyze_participle;
//!
//! let p = analyze_participle("γραφων").unwrap();
//! assert_eq!(p.verb_lemma(), "γραφω");
//! assert_eq!(p.tense, Tense::Present);
//! ```

use super::{Case, Number, Gender, Tense, Voice};

/// Result of participle morphological analysis
#[derive(Debug, Clone, PartialEq)]
pub struct ParticipleAnalysis {
    /// The verb stem (without participle suffix)
    pub stem: String,
    /// Tense of the participle
    pub tense: Tense,
    /// Voice of the participle
    pub voice: Voice,
    /// Case (adjectival property)
    pub case: Case,
    /// Gender (adjectival property)
    pub gender: Gender,
    /// Number (adjectival property)
    pub number: Number,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

impl ParticipleAnalysis {
    /// Get the lemma (dictionary form) of the verb
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let p = analyze_participle("γραφων").unwrap();
    /// assert_eq!(p.verb_lemma(), "γραφω");
    /// ```
    pub fn verb_lemma(&self) -> String {
        format!("{}ω", self.stem)
    }
}

/// Participle ending pattern
#[derive(Debug)]
struct ParticiplePattern {
    ending: &'static str,
    tense: Tense,
    voice: Voice,
    case: Case,
    gender: Gender,
    number: Number,
}

/// Present Active Participle endings (-ων, -ουσα, -ον)
/// These are third declension (masc/neut) and first declension (fem)
const PRESENT_ACTIVE_PARTICIPLE: &[ParticiplePattern] = &[
    // Masculine singular
    ParticiplePattern {
        ending: "ων",
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "οντος",
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Genitive,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "οντι",
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Dative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "οντα",
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Accusative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    // Feminine singular (-ουσα, -ουσης, -ουσῃ, -ουσαν)
    ParticiplePattern {
        ending: "ουσα",
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Feminine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "ουσης",
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Genitive,
        gender: Gender::Feminine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "ουση",
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Dative,
        gender: Gender::Feminine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "ουσαν",
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Accusative,
        gender: Gender::Feminine,
        number: Number::Singular,
    },
    // Neuter singular (-ον, -οντος, -οντι)
    ParticiplePattern {
        ending: "ον",
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Neuter,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "ον",
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Accusative,
        gender: Gender::Neuter,
        number: Number::Singular,
    },
    // Masculine plural
    ParticiplePattern {
        ending: "οντες",
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Masculine,
        number: Number::Plural,
    },
    ParticiplePattern {
        ending: "οντων",
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Genitive,
        gender: Gender::Masculine,
        number: Number::Plural,
    },
    // Neuter plural
    ParticiplePattern {
        ending: "οντα",
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Neuter,
        number: Number::Plural,
    },
];

/// Present Middle/Passive Participle endings (-ομενος, -ομενη, -ομενον)
/// These follow second/first declension patterns
/// Note: Neuter forms listed before masculine for ambiguous -ομενον (more common in lambdas)
const PRESENT_MIDDLE_PARTICIPLE: &[ParticiplePattern] = &[
    // Masculine singular (unique forms)
    ParticiplePattern {
        ending: "ομενος",
        tense: Tense::Present,
        voice: Voice::Middle,
        case: Case::Nominative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "ομενου",
        tense: Tense::Present,
        voice: Voice::Middle,
        case: Case::Genitive,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "ομενῳ",
        tense: Tense::Present,
        voice: Voice::Middle,
        case: Case::Dative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "ομενω",  // normalized form (iota subscript removed)
        tense: Tense::Present,
        voice: Voice::Middle,
        case: Case::Dative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    // Plural forms
    ParticiplePattern {
        ending: "ομενοι",
        tense: Tense::Present,
        voice: Voice::Middle,
        case: Case::Nominative,
        gender: Gender::Masculine,
        number: Number::Plural,
    },
    ParticiplePattern {
        ending: "ομενα",
        tense: Tense::Present,
        voice: Voice::Middle,
        case: Case::Nominative,
        gender: Gender::Neuter,
        number: Number::Plural,
    },
    // Feminine singular
    ParticiplePattern {
        ending: "ομενη",
        tense: Tense::Present,
        voice: Voice::Middle,
        case: Case::Nominative,
        gender: Gender::Feminine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "ομενης",
        tense: Tense::Present,
        voice: Voice::Middle,
        case: Case::Genitive,
        gender: Gender::Feminine,
        number: Number::Singular,
    },
    // Neuter singular (preferred for ambiguous -ομενον)
    ParticiplePattern {
        ending: "ομενον",
        tense: Tense::Present,
        voice: Voice::Middle,
        case: Case::Nominative,
        gender: Gender::Neuter,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "ομενον",
        tense: Tense::Present,
        voice: Voice::Middle,
        case: Case::Accusative,
        gender: Gender::Neuter,
        number: Number::Singular,
    },
    // Masculine accusative (after neuter to give neuter priority)
    ParticiplePattern {
        ending: "ομενον",
        tense: Tense::Present,
        voice: Voice::Middle,
        case: Case::Accusative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
];

/// Aorist Active Participle endings (-ας, -ασα, -αν)
/// Note: The aorist tense marker (σ) is included in the stem, causing phonological changes
/// (φσ → ψ, κσ → ξ, τσ → σ, etc.). The pure participial endings are -ας, -ασα, -αν, -αντ-.
const AORIST_ACTIVE_PARTICIPLE: &[ParticiplePattern] = &[
    // Longer endings first for greedy matching
    // Note: The σ tense marker may merge with stem (φσ→ψ, κσ→ξ, etc.)
    ParticiplePattern {
        ending: "αντος",
        tense: Tense::Aorist,
        voice: Voice::Active,
        case: Case::Genitive,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "αντες",
        tense: Tense::Aorist,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Masculine,
        number: Number::Plural,
    },
    ParticiplePattern {
        ending: "αντα",
        tense: Tense::Aorist,
        voice: Voice::Active,
        case: Case::Accusative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "αντα",
        tense: Tense::Aorist,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Neuter,
        number: Number::Plural,
    },
    ParticiplePattern {
        ending: "αντι",
        tense: Tense::Aorist,
        voice: Voice::Active,
        case: Case::Dative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "ασης",
        tense: Tense::Aorist,
        voice: Voice::Active,
        case: Case::Genitive,
        gender: Gender::Feminine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "ασα",
        tense: Tense::Aorist,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Feminine,
        number: Number::Singular,
    },
    // Masculine nominative singular
    // Note: phonological changes (φσ→ψ, κσ→ξ) occur in the stem, not the ending
    ParticiplePattern {
        ending: "ας",
        tense: Tense::Aorist,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    // Neuter singular
    ParticiplePattern {
        ending: "αν",
        tense: Tense::Aorist,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Neuter,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "αν",
        tense: Tense::Aorist,
        voice: Voice::Active,
        case: Case::Accusative,
        gender: Gender::Neuter,
        number: Number::Singular,
    },
];

/// Perfect Passive Participle endings (-μενος, -μενη, -μενον)
/// Note: Perfect participles often have reduplication in the stem
const PERFECT_PASSIVE_PARTICIPLE: &[ParticiplePattern] = &[
    // Masculine singular
    ParticiplePattern {
        ending: "μενος",
        tense: Tense::Perfect,
        voice: Voice::Passive,
        case: Case::Nominative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "μενου",
        tense: Tense::Perfect,
        voice: Voice::Passive,
        case: Case::Genitive,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "μενῳ",
        tense: Tense::Perfect,
        voice: Voice::Passive,
        case: Case::Dative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "μενω",  // normalized (iota subscript removed)
        tense: Tense::Perfect,
        voice: Voice::Passive,
        case: Case::Dative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "μενον",
        tense: Tense::Perfect,
        voice: Voice::Passive,
        case: Case::Accusative,
        gender: Gender::Masculine,
        number: Number::Singular,
    },
    // Feminine singular
    ParticiplePattern {
        ending: "μενη",
        tense: Tense::Perfect,
        voice: Voice::Passive,
        case: Case::Nominative,
        gender: Gender::Feminine,
        number: Number::Singular,
    },
    // Neuter singular
    ParticiplePattern {
        ending: "μενον",
        tense: Tense::Perfect,
        voice: Voice::Passive,
        case: Case::Nominative,
        gender: Gender::Neuter,
        number: Number::Singular,
    },
    ParticiplePattern {
        ending: "μενον",
        tense: Tense::Perfect,
        voice: Voice::Passive,
        case: Case::Accusative,
        gender: Gender::Neuter,
        number: Number::Singular,
    },
    // Plural forms
    ParticiplePattern {
        ending: "μενοι",
        tense: Tense::Perfect,
        voice: Voice::Passive,
        case: Case::Nominative,
        gender: Gender::Masculine,
        number: Number::Plural,
    },
    ParticiplePattern {
        ending: "μενα",
        tense: Tense::Perfect,
        voice: Voice::Passive,
        case: Case::Nominative,
        gender: Gender::Neuter,
        number: Number::Plural,
    },
];

/// Analyze a Greek word as a participle
///
/// Attempts to match the word against known participle ending patterns.
/// Returns `Some(ParticipleAnalysis)` if the word appears to be a participle,
/// `None` otherwise.
///
/// # Examples
///
/// ```ignore
/// use glossa::morphology::participle::analyze_participle;
///
/// let p = analyze_participle("γραφων").unwrap();
/// assert_eq!(p.verb_lemma(), "γραφω");
/// assert_eq!(p.tense, Tense::Present);
/// ```
pub fn analyze_participle(word: &str) -> Option<ParticipleAnalysis> {
    // Combine all pattern tables into a single vector
    let mut all_patterns: Vec<&ParticiplePattern> = Vec::new();
    all_patterns.extend(PRESENT_ACTIVE_PARTICIPLE.iter());
    all_patterns.extend(PRESENT_MIDDLE_PARTICIPLE.iter());
    all_patterns.extend(AORIST_ACTIVE_PARTICIPLE.iter());
    all_patterns.extend(PERFECT_PASSIVE_PARTICIPLE.iter());

    // Sort ALL patterns by ending length (longest first) for greedy matching
    // This ensures "ομενον" is matched before "ον"
    all_patterns.sort_by(|a, b| b.ending.len().cmp(&a.ending.len()));

    for pattern in all_patterns {
        if word.ends_with(pattern.ending) {
            let stem = &word[..word.len() - pattern.ending.len()];

            // Stem must not be empty
            if stem.is_empty() {
                continue;
            }

            // For perfect participles, check for reduplication
            // (This is a heuristic - perfect stems often start with repeated consonant)
            let confidence = if pattern.tense == Tense::Perfect {
                if has_reduplication(stem) {
                    0.85
                } else {
                    0.65  // Lower confidence without reduplication
                }
            } else {
                0.80
            };

            return Some(ParticipleAnalysis {
                stem: stem.to_string(),
                tense: pattern.tense,
                voice: pattern.voice,
                case: pattern.case,
                gender: pattern.gender,
                number: pattern.number,
                confidence,
            });
        }
    }

    None
}

/// Check if a stem shows signs of reduplication (perfect tense marker)
///
/// Greek perfect stems often have reduplication:
/// - γεγραφ- (γραφω)
/// - λελυκ- (λυω)
/// - πεπαυκ- (παυω)
fn has_reduplication(stem: &str) -> bool {
    if stem.len() < 2 {
        return false;
    }

    let chars: Vec<char> = stem.chars().collect();

    // Check for simple reduplication: first two letters are identical or similar
    // γε-γραφ, λε-λυκ, πε-παυκ
    if chars.len() >= 4 {
        let first = chars[0];
        let second = chars[1];
        let third = chars[2];

        // Perfect reduplication patterns:
        // 1. Consonant + ε + same consonant (γε-γ, λε-λ, πε-π)
        if second == 'ε' && first == third {
            return true;
        }

        // 2. Simple vowel reduplication (ἠ-ακ from ἀκουω)
        if first == second && is_greek_vowel(first) {
            return true;
        }
    }

    false
}

/// Check if a character is a Greek vowel
fn is_greek_vowel(c: char) -> bool {
    matches!(c, 'α' | 'ε' | 'η' | 'ι' | 'ο' | 'υ' | 'ω')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_present_active_participle_masculine() {
        let p = analyze_participle("γραφων").unwrap();
        assert_eq!(p.tense, Tense::Present);
        assert_eq!(p.voice, Voice::Active);
        assert_eq!(p.gender, Gender::Masculine);
        assert_eq!(p.case, Case::Nominative);
        assert_eq!(p.number, Number::Singular);
        assert_eq!(p.stem, "γραφ");
    }

    #[test]
    fn test_present_active_participle_feminine() {
        let p = analyze_participle("γραφουσα").unwrap();
        assert_eq!(p.tense, Tense::Present);
        assert_eq!(p.voice, Voice::Active);
        assert_eq!(p.gender, Gender::Feminine);
    }

    #[test]
    fn test_present_active_participle_neuter() {
        let p = analyze_participle("γραφον").unwrap();
        assert_eq!(p.tense, Tense::Present);
        assert_eq!(p.voice, Voice::Active);
        assert_eq!(p.gender, Gender::Neuter);
    }

    #[test]
    fn test_present_middle_participle() {
        let p = analyze_participle("διπλασιαζομενον").unwrap();
        assert_eq!(p.tense, Tense::Present);
        assert_eq!(p.voice, Voice::Middle);
        assert_eq!(p.gender, Gender::Neuter);
        assert_eq!(p.stem, "διπλασιαζ");
    }

    #[test]
    fn test_aorist_active_participle() {
        let p = analyze_participle("γραψας").unwrap();
        assert_eq!(p.tense, Tense::Aorist);
        assert_eq!(p.voice, Voice::Active);
        assert_eq!(p.gender, Gender::Masculine);
        assert_eq!(p.stem, "γραψ");
    }

    #[test]
    fn test_aorist_feminine() {
        let p = analyze_participle("γραψασα").unwrap();
        assert_eq!(p.tense, Tense::Aorist);
        assert_eq!(p.gender, Gender::Feminine);
    }

    #[test]
    fn test_aorist_neuter() {
        let p = analyze_participle("γραψαν").unwrap();
        assert_eq!(p.tense, Tense::Aorist);
        assert_eq!(p.gender, Gender::Neuter);
    }

    #[test]
    fn test_perfect_passive_participle() {
        let p = analyze_participle("γεγραμμενος").unwrap();
        assert_eq!(p.tense, Tense::Perfect);
        assert_eq!(p.voice, Voice::Passive);
        assert_eq!(p.stem, "γεγραμ");
    }

    #[test]
    fn test_verb_lemma() {
        let p = analyze_participle("γραφων").unwrap();
        assert_eq!(p.verb_lemma(), "γραφω");
    }

    #[test]
    fn test_reduplication_detection() {
        assert!(has_reduplication("γεγραφ"));
        assert!(has_reduplication("λελυκ"));
        assert!(has_reduplication("πεπαυκ"));
        assert!(!has_reduplication("γραφ"));
        assert!(!has_reduplication("λυ"));
    }
}
