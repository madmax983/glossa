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
//! ```
//! use glossa::morphology::participle::analyze_participle;
//! use glossa::morphology::Tense;
//!
//! let p = analyze_participle("γραφων").unwrap();
//! assert_eq!(p.verb_lemma(), "γραφω");
//! assert_eq!(p.tense, Tense::Present);
//! ```

use super::{Case, Gender, Number, Tense, Voice};
use std::sync::LazyLock;

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
    /// ```
    /// use glossa::morphology::participle::analyze_participle;
    ///
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
        ending: "ομενω", // normalized form (iota subscript removed)
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
        ending: "μενω", // normalized (iota subscript removed)
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

/// Combined and sorted list of all participle patterns
///
/// Pre-computed once to avoid allocation and sorting on every call.
static ALL_PATTERNS: LazyLock<Vec<&'static ParticiplePattern>> = LazyLock::new(|| {
    let mut patterns = Vec::new();
    patterns.extend(PRESENT_ACTIVE_PARTICIPLE.iter());
    patterns.extend(PRESENT_MIDDLE_PARTICIPLE.iter());
    patterns.extend(AORIST_ACTIVE_PARTICIPLE.iter());
    patterns.extend(PERFECT_PASSIVE_PARTICIPLE.iter());

    // Sort by ending length (longest first) for greedy matching
    // This ensures "ομενον" is matched before "ον"
    patterns.sort_by(|a, b| b.ending.len().cmp(&a.ending.len()));

    patterns
});

/// Analyze a Greek word as a participle
///
/// Attempts to match the word against known participle ending patterns.
/// Returns `Some(ParticipleAnalysis)` if the word appears to be a participle,
/// `None` otherwise.
///
/// # Examples
///
/// ```
/// use glossa::morphology::participle::analyze_participle;
/// use glossa::morphology::Tense;
///
/// let p = analyze_participle("γραφων").unwrap();
/// assert_eq!(p.verb_lemma(), "γραφω");
/// assert_eq!(p.tense, Tense::Present);
/// ```
pub fn analyze_participle(word: &str) -> Option<ParticipleAnalysis> {
    for pattern in ALL_PATTERNS.iter() {
        if let Some(stem) = word.strip_suffix(pattern.ending) {
            // Stem must not be empty
            if stem.is_empty() {
                continue;
            }

            return Some(ParticipleAnalysis {
                stem: stem.to_string(),
                tense: pattern.tense,
                voice: pattern.voice,
                case: pattern.case,
                gender: pattern.gender,
                number: pattern.number,
                confidence: 0.9,
            });
        }
    }

    None
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
}
