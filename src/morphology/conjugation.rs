//! Verb conjugation tables and analysis
//!
//! Implements verb analysis for ΓΛΩΣΣΑ, focusing on:
//! - Present Active Indicative (streaming operations)
//! - Aorist Active Indicative (one-shot operations)
//! - Imperative (commands)

use std::borrow::Cow;

use crate::morphology::matcher::match_suffix;
use crate::morphology::models::{Mood, MorphAnalysis, Number, PartOfSpeech, Person, Tense, Voice};
use crate::text::normalize_greek;

/// Present Active Indicative endings (ω-conjugation)
/// Pattern: λέγω, γράφω, etc.
/// NOTE: Must be sorted by length descending!
const PRESENT_ACTIVE_IND: &[(&str, Person, Number)] = &[
    ("ουσιν", Person::Third, Number::Plural), // with movable nu
    ("ομεν", Person::First, Number::Plural),
    ("ουσι", Person::Third, Number::Plural),
    ("εις", Person::Second, Number::Singular),
    ("ετε", Person::Second, Number::Plural),
    ("ει", Person::Third, Number::Singular),
    ("ω", Person::First, Number::Singular),
];

/// Present Active Imperative endings
/// NOTE: Must be sorted by length descending!
const PRESENT_ACTIVE_IMP: &[(&str, Person, Number)] = &[
    ("οντων", Person::Third, Number::Plural),
    ("ετε", Person::Second, Number::Plural),
    ("ετω", Person::Third, Number::Singular),
    ("ε", Person::Second, Number::Singular),
];

/// Aorist Active Indicative endings (first/sigmatic aorist)
/// Pattern: ἔλυσα, ἔγραψα, etc.
/// NOTE: Must be sorted by length descending!
const AORIST_ACTIVE_IND: &[(&str, Person, Number)] = &[
    ("σαμεν", Person::First, Number::Plural),
    ("σατε", Person::Second, Number::Plural),
    ("σαν", Person::Third, Number::Plural),
    ("σας", Person::Second, Number::Singular),
    ("σεν", Person::Third, Number::Singular), // with movable nu
    ("σα", Person::First, Number::Singular),
    ("σε", Person::Third, Number::Singular),
];

/// Aorist Active Imperative endings
/// NOTE: Must be sorted by length descending!
const AORIST_ACTIVE_IMP: &[(&str, Person, Number)] = &[
    ("σαντων", Person::Third, Number::Plural),
    ("σατε", Person::Second, Number::Plural),
    ("σατω", Person::Third, Number::Singular),
    ("σον", Person::Second, Number::Singular),
];

/// Present Active Subjunctive endings (long vowel theme)
/// Pattern: λύω → λύω (identical to present indicative in form, but subjunctive in meaning)
/// The subjunctive has lengthened thematic vowel: ω/η instead of ο/ε
/// NOTE: Must be sorted by length descending!
const PRESENT_ACTIVE_SUBJ: &[(&str, Person, Number)] = &[
    ("ωμεν", Person::First, Number::Plural),
    ("ωσιν", Person::Third, Number::Plural), // with movable nu
    ("ητε", Person::Second, Number::Plural),
    ("ωσι", Person::Third, Number::Plural),
    ("ης", Person::Second, Number::Singular), // Note: η + ς (normalized)
    ("η", Person::Third, Number::Singular),   // Long vowel ῃ (normalized)
    ("ω", Person::First, Number::Singular),
];

/// Aorist Active Subjunctive endings
/// Pattern: λύσω (σ + subjunctive endings)
/// NOTE: Must be sorted by length descending!
const AORIST_ACTIVE_SUBJ: &[(&str, Person, Number)] = &[
    ("σωμεν", Person::First, Number::Plural),
    ("σωσιν", Person::Third, Number::Plural),
    ("σητε", Person::Second, Number::Plural),
    ("σωσι", Person::Third, Number::Plural),
    ("σης", Person::Second, Number::Singular),
    ("ση", Person::Third, Number::Singular),
    ("σω", Person::First, Number::Singular),
];

/// Present Active Optative endings
/// Pattern: γράφοιμι "I might write"
/// The optative mood expresses possibility, wish, or potential - natural for `Option<T>`
/// NOTE: Must be sorted by length descending!
const PRESENT_ACTIVE_OPT: &[(&str, Person, Number)] = &[
    ("οιμεν", Person::First, Number::Plural),
    ("οιεν", Person::Third, Number::Plural),
    ("οιμι", Person::First, Number::Singular),
    ("οιτε", Person::Second, Number::Plural),
    ("οις", Person::Second, Number::Singular),
    ("οι", Person::Third, Number::Singular),
];

/// Aorist Passive Optative endings
/// Pattern: εὑρεθείη "might be found"
/// Used for values that "might exist" (`Option<T>` semantics)
/// NOTE: Must be sorted by length descending!
const AORIST_PASSIVE_OPT: &[(&str, Person, Number)] = &[
    ("θειημεν", Person::First, Number::Plural),
    ("θειησαν", Person::Third, Number::Plural),
    ("θειητε", Person::Second, Number::Plural),
    ("θειην", Person::First, Number::Singular),
    ("θειης", Person::Second, Number::Singular),
    ("θειη", Person::Third, Number::Singular),
];

/// Present Active Infinitive ending
const PRESENT_INFINITIVE: &str = "ειν";

/// Aorist Active Infinitive ending
const AORIST_INFINITIVE: &str = "σαι";

/// Known irregular aorist stems and their present stems
/// This handles verbs where heuristic augment stripping would fail
///
/// Format: (augmented_stem_prefix, present_stem)
/// The key is matched at the START of the augmented stem
const IRREGULAR_AORISTS: &[(&str, &str)] = &[
    // η-augment verbs (α → η) - note: ἤγαγον has reduplication too
    // For ἤγαγον: stem is αγαγ (reduplicated), augment makes η. Lemma: αγω
    // But we only strip augment here, not reduplication, so αγαγ → αγω is handled by the
    // lemma formation adding -ω to the stem
    ("ηκουσ", "ακου"), // ἀκούω → ἤκουσα
    ("ηρξ", "αρχ"),    // ἄρχω → ἦρξα
    // ε-augment that might look like stem
    ("εθελ", "θελ"), // θέλω → ἠθέλησα (irregular)
    // ω-augment verbs (ο → ω)
    ("ωνομασ", "ονομαζ"), // ὀνομάζω → ὠνόμασα
];

/// Verbs that naturally start with ε (don't strip!)
const VERBS_STARTING_WITH_EPSILON: &[&str] = &[
    "εχ",    // ἔχω (to have)
    "ελπιζ", // ἐλπίζω (to hope)
    "εργαζ", // ἐργάζομαι (to work)
    "εστι",  // εἶναι forms
];

/// Strip the temporal augment from an aorist stem to find the true verb stem
///
/// The augment marks past tense in Greek:
/// - Simple ε-augment: ἔλυσα → λυ- (ε + λυ + σα)
/// - Vowel lengthening: ἤγαγον → ἀγ- (α → η)
///
/// Without stripping, "ελυσα" would give lemma "ελυω" instead of "λυω"
///
/// Note: Input should already be normalized (monotonic, lowercase)
fn strip_augment(augmented_stem: &str) -> Cow<'_, str> {
    // First, check the irregular aorists lookup table
    for (aorist, present) in IRREGULAR_AORISTS {
        if augmented_stem.starts_with(*aorist) {
            let rest = &augmented_stem[aorist.len()..];
            return Cow::Owned(format!("{}{}", present, rest));
        }
    }

    // Check if this verb naturally starts with ε (don't strip!)
    for prefix in VERBS_STARTING_WITH_EPSILON {
        if augmented_stem.starts_with(*prefix) {
            return Cow::Borrowed(augmented_stem);
        }
    }

    // Simple epsilon augment (most common)
    // After normalization, this is just "ε"
    if let Some(stripped) = augmented_stem.strip_prefix("ε") {
        // Make sure we're not stripping from a vowel-initial stem
        // that received ε → η or ε → ει augment
        if !stripped.is_empty() {
            return Cow::Borrowed(stripped);
        }
    }

    // Vowel lengthening augments (less common, but important)
    // α → η (e.g., ἄγω → ἤγαγον)
    if let Some(rest) = augmented_stem.strip_prefix("η") {
        // Could be α → η augment, restore α
        // But η could also be original (1st decl stem), so this is heuristic
        if !rest.is_empty() {
            return Cow::Owned(format!("α{}", rest));
        }
    }

    // ο → ω (e.g., ὀνομάζω → ὠνόμασα)
    if let Some(rest) = augmented_stem.strip_prefix("ω")
        && !rest.is_empty()
    {
        return Cow::Owned(format!("ο{}", rest));
    }

    // ε → η (ε-contract verbs)
    // αι → ῃ, ει → ῃ, οι → ῳ - these are rarer
    // For MVP, we handle the common cases

    // No augment found, return as-is
    Cow::Borrowed(augmented_stem)
}

/// Strategy for lemma generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LemmaStrategy {
    Standard,
    StripAugment,
    PassiveOptative,
}

/// Helper struct for conjugation patterns
struct ConjugationPattern {
    endings: &'static [(&'static str, Person, Number)],
    tense: Tense,
    mood: Mood,
    voice: Voice,
    lemma_strategy: LemmaStrategy,
    base_confidence: f32,
}

/// Ordered list of conjugation patterns for analysis.
/// Order matters for `analyze_verb` (first match wins).
const CONJUGATION_PATTERNS: &[ConjugationPattern] = &[
    ConjugationPattern {
        endings: PRESENT_ACTIVE_IND,
        tense: Tense::Present,
        mood: Mood::Indicative,
        voice: Voice::Active,
        lemma_strategy: LemmaStrategy::Standard,
        base_confidence: 0.8,
    },
    ConjugationPattern {
        endings: PRESENT_ACTIVE_IMP,
        tense: Tense::Present,
        mood: Mood::Imperative,
        voice: Voice::Active,
        lemma_strategy: LemmaStrategy::Standard,
        base_confidence: 0.75,
    },
    ConjugationPattern {
        endings: AORIST_ACTIVE_IND,
        tense: Tense::Aorist,
        mood: Mood::Indicative,
        voice: Voice::Active,
        lemma_strategy: LemmaStrategy::StripAugment,
        base_confidence: 0.85,
    },
    ConjugationPattern {
        endings: AORIST_ACTIVE_IMP,
        tense: Tense::Aorist,
        mood: Mood::Imperative,
        voice: Voice::Active,
        lemma_strategy: LemmaStrategy::Standard,
        base_confidence: 0.75,
    },
    ConjugationPattern {
        endings: AORIST_PASSIVE_OPT,
        tense: Tense::Aorist,
        mood: Mood::Optative,
        voice: Voice::Passive,
        lemma_strategy: LemmaStrategy::PassiveOptative,
        base_confidence: 0.75,
    },
    ConjugationPattern {
        endings: PRESENT_ACTIVE_SUBJ,
        tense: Tense::Present,
        mood: Mood::Subjunctive,
        voice: Voice::Active,
        lemma_strategy: LemmaStrategy::Standard,
        base_confidence: 0.70,
    },
    ConjugationPattern {
        endings: AORIST_ACTIVE_SUBJ,
        tense: Tense::Aorist,
        mood: Mood::Subjunctive,
        voice: Voice::Active,
        lemma_strategy: LemmaStrategy::Standard,
        base_confidence: 0.70,
    },
    ConjugationPattern {
        endings: PRESENT_ACTIVE_OPT,
        tense: Tense::Present,
        mood: Mood::Optative,
        voice: Voice::Active,
        lemma_strategy: LemmaStrategy::Standard,
        base_confidence: 0.75,
    },
];

/// Subjunctive forms of εἰμί (to be) - irregular but essential for conditionals
const EIMI_SUBJUNCTIVE: &[(&str, Person, Number)] = &[
    ("ω", Person::First, Number::Singular),   // ὦ
    ("ης", Person::Second, Number::Singular), // ᾖς (normalized)
    ("η", Person::Third, Number::Singular),   // ᾖ (normalized) - most common in conditionals
    ("ωμεν", Person::First, Number::Plural),  // ὦμεν
    ("ητε", Person::Second, Number::Plural),  // ἦτε
    ("ωσι", Person::Third, Number::Plural),   // ὦσι
    ("ωσιν", Person::Third, Number::Plural),  // with movable nu
];

/// Try to analyze a word as a verb
///
/// Returns the most likely morphological analysis for the given verb form.
/// This function checks various conjugation patterns including:
/// - Present Active Indicative (-ω)
/// - Present Active Imperative (-ε)
/// - Aorist Active Indicative (-σα)
/// - Aorist Active Imperative (-σον)
/// - Infinitives (-ειν, -σαι)
/// - Subjunctives and Optatives
///
/// # Examples
///
/// ```
/// use glossa::morphology::{analyze_verb, Tense, Mood, Person, Number};
///
/// // Present Indicative
/// let analysis = analyze_verb("λέγω").unwrap();
/// assert_eq!(analysis.tense, Some(Tense::Present));
/// assert_eq!(analysis.mood, Some(Mood::Indicative));
///
/// // Aorist Indicative (with augment stripping)
/// // ἔλυσα -> lemma λυω
/// let analysis = analyze_verb("ἔλυσα").unwrap();
/// assert_eq!(analysis.tense, Some(Tense::Aorist));
/// assert_eq!(analysis.lemma, "λυω");
///
/// // Imperative
/// let analysis = analyze_verb("λέγε").unwrap();
/// assert_eq!(analysis.mood, Some(Mood::Imperative));
/// ```
pub fn analyze_verb(word: &str) -> Option<MorphAnalysis> {
    let word_string = normalize_greek(word);
    let word = word_string.as_str();

    // Special handling for εἰμί (to be) subjunctive forms
    // These are irregular and essential for conditionals (εἰ ... ᾖ)
    for (form, person, number) in EIMI_SUBJUNCTIVE {
        if word == *form {
            return Some(MorphAnalysis {
                lemma: Cow::Borrowed("ειμι"),
                part_of_speech: PartOfSpeech::Verb,
                case: None,
                number: Some(*number),
                gender: None,
                person: Some(*person),
                tense: Some(Tense::Present),
                mood: Some(Mood::Subjunctive),
                voice: Some(Voice::Active),
                confidence: 0.95,
            });
        }
    }

    // Try conjugation patterns in order
    for pattern in CONJUGATION_PATTERNS {
        if let Some(analysis) = try_conjugation_pattern(word, pattern) {
            return Some(analysis);
        }
    }

    // Try present infinitive
    if let Some(stem) = word.strip_suffix(PRESENT_INFINITIVE)
        && !stem.is_empty()
    {
        return Some(MorphAnalysis {
            lemma: Cow::Owned(format!("{}ω", stem)),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: Some(Tense::Present),
            mood: Some(Mood::Infinitive),
            voice: Some(Voice::Active),
            confidence: 0.85,
        });
    }

    // Try aorist infinitive
    if let Some(stem) = word.strip_suffix(AORIST_INFINITIVE)
        && !stem.is_empty()
    {
        return Some(MorphAnalysis {
            lemma: Cow::Owned(format!("{}ω", stem)),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: Some(Tense::Aorist),
            mood: Some(Mood::Infinitive),
            voice: Some(Voice::Active),
            confidence: 0.85,
        });
    }

    None
}

fn try_conjugation_pattern_all(
    word: &str,
    pattern: &ConjugationPattern,
    analyses: &mut Vec<MorphAnalysis>,
) {
    match_verb_endings_all(word, pattern.endings, |stem, person, number| {
        // Handle lemma generation
        let lemma_stem: Cow<str> = match pattern.lemma_strategy {
            LemmaStrategy::Standard => Cow::Borrowed(stem),
            LemmaStrategy::StripAugment => strip_augment(stem),
            LemmaStrategy::PassiveOptative => Cow::Borrowed(stem.strip_suffix('θ').unwrap_or(stem)),
        };

        // Calculate confidence
        let ending_len = word.len() - stem.len();
        let length_bonus = (ending_len as f32 - 1.0) * 0.03;
        let confidence = (pattern.base_confidence + length_bonus).min(0.95);

        // Optimization: If lemma_stem is just stem (Standard strategy), and ending is "ω", then lemma == word
        let lemma = if matches!(pattern.lemma_strategy, LemmaStrategy::Standard)
            && word.ends_with('ω')
            && word.len() == stem.len() + "ω".len()
        {
            Cow::Owned(word.to_string())
        } else {
            Cow::Owned(format!("{}ω", lemma_stem))
        };

        analyses.push(MorphAnalysis {
            lemma,
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(number),
            gender: None,
            person: Some(person),
            tense: Some(pattern.tense),
            mood: Some(pattern.mood),
            voice: Some(pattern.voice),
            confidence,
        });
    });
}

fn try_conjugation_pattern(word: &str, pattern: &ConjugationPattern) -> Option<MorphAnalysis> {
    let (stem, person, number) = match_verb_endings(word, pattern.endings)?;

    let lemma = match pattern.lemma_strategy {
        LemmaStrategy::Standard => {
            // Optimization: If ending is "ω", then lemma == word (e.g. λέγω)
            if word.ends_with('ω') && word.len() == stem.len() + "ω".len() {
                Cow::Owned(word.to_string())
            } else {
                Cow::Owned(format!("{}ω", stem))
            }
        }
        LemmaStrategy::StripAugment => {
            let true_stem = strip_augment(stem);
            Cow::Owned(format!("{}ω", true_stem))
        }
        LemmaStrategy::PassiveOptative => {
            // Strip the θη passive marker (if present) to get base stem, then add -ω for lemma
            // Use strip_suffix instead of trim_end_matches to avoid over-stripping root characters
            let base_stem = stem.strip_suffix('θ').unwrap_or(stem);
            Cow::Owned(format!("{}ω", base_stem))
        }
    };

    Some(MorphAnalysis {
        lemma,
        part_of_speech: PartOfSpeech::Verb,
        case: None,
        number: Some(number),
        gender: None,
        person: Some(person),
        tense: Some(pattern.tense),
        mood: Some(pattern.mood),
        voice: Some(pattern.voice),
        confidence: pattern.base_confidence,
    })
}

/// Match a word against verb endings
///
/// Note: The `endings` slice MUST be sorted by length descending.
fn match_verb_endings<'a>(
    word: &'a str,
    endings: &[(&str, Person, Number)],
) -> Option<(&'a str, Person, Number)> {
    let mut result = None;
    match_suffix(
        word,
        endings,
        |e| e.0,
        |stem, pattern| {
            if result.is_none() {
                result = Some((stem, pattern.1, pattern.2));
                false // Stop after first match
            } else {
                true // Should not happen if we stop, but safe default
            }
        },
    );
    result
}

/// Match a word against ALL verb endings (for ambiguity resolution)
fn match_verb_endings_all<F>(word: &str, endings: &[(&str, Person, Number)], mut callback: F)
where
    F: FnMut(&str, Person, Number),
{
    match_suffix(
        word,
        endings,
        |e| e.0,
        |stem, pattern| {
            callback(stem, pattern.1, pattern.2);
            true // Continue searching
        },
    );
}

/// Analyze a word as a verb, returning ALL possible analyses
///
/// This handles cases where a form could belong to multiple paradigms.
/// For example, "-ε" could be:
/// - 2nd person singular present imperative (λέγε!)
/// - 3rd person singular aorist indicative (ἔλυσε)
pub fn analyze_verb_all(word: &str) -> Vec<MorphAnalysis> {
    let mut analyses = Vec::with_capacity(8);
    analyze_verb_all_into(word, &mut analyses);
    analyses
}

/// Analyze a word as a verb, pushing results into an existing vector
///
/// Zero-allocation version of `analyze_verb_all`.
fn try_analyze_infinitive(
    word: &str,
    suffix: &str,
    tense: Tense,
    analyses: &mut Vec<MorphAnalysis>,
) {
    if let Some(stem) = word.strip_suffix(suffix)
        && !stem.is_empty()
    {
        analyses.push(MorphAnalysis {
            lemma: Cow::Owned(format!("{}ω", stem)),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: Some(tense),
            mood: Some(Mood::Infinitive),
            voice: Some(Voice::Active),
            confidence: 0.85,
        });
    }
}

/// Analyze a verb and append all possible morphological interpretations to an existing vector.
///
/// This exists to allow callers to collect analyses without allocating a new `Vec`
/// each time. By passing in a mutable reference to an existing collection, you can
/// accumulate results across multiple words or fallback strategies efficiently.
///
/// ## Examples
///
/// ```text
/// let mut analyses = Vec::with_capacity(8);
/// // Append all analyses of "γράφω" into our vector
/// analyze_verb_all_into("γραφω", &mut analyses);
/// assert!(!analyses.is_empty());
/// ```
pub fn analyze_verb_all_into(word: &str, analyses: &mut Vec<MorphAnalysis>) {
    let start_len = analyses.len();

    // Special handling for εἰμί (to be) subjunctive forms
    // These are irregular and essential for conditionals (εἰ ... ᾖ)
    for (form, person, number) in EIMI_SUBJUNCTIVE {
        if word == *form {
            analyses.push(MorphAnalysis {
                lemma: Cow::Borrowed("ειμι"),
                part_of_speech: PartOfSpeech::Verb,
                case: None,
                number: Some(*number),
                gender: None,
                person: Some(*person),
                tense: Some(Tense::Present),
                mood: Some(Mood::Subjunctive),
                voice: Some(Voice::Active),
                confidence: 0.95,
            });
        }
    }

    for pattern in CONJUGATION_PATTERNS {
        try_conjugation_pattern_all(word, pattern, analyses);
    }

    try_analyze_infinitive(word, PRESENT_INFINITIVE, Tense::Present, analyses);
    try_analyze_infinitive(word, AORIST_INFINITIVE, Tense::Aorist, analyses);

    // Sort the newly added analyses (the tail)
    analyses[start_len..].sort_by(|a, b| {
        let key_a = (a.tense, a.mood, a.person, a.number, &a.lemma);
        let key_b = (b.tense, b.mood, b.person, b.number, &b.lemma);
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
                a.tense == b.tense
                    && a.mood == b.mood
                    && a.person == b.person
                    && a.number == b.number
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

/// Conjugate a verb stem to a specific form
///
/// Reconstructs the specified form of a verb based on its stem and morphological features.
/// This is particularly useful for generating Greek output or error messages that require
/// correctly inflected verbs.
///
/// # Examples
///
/// ```rust
/// use glossa::morphology::{conjugate, Tense, Mood, Voice, Person, Number};
///
/// // Conjugate "λεγ" (to say) in Present Active Indicative, 1st Person Singular
/// let result = conjugate("λεγ", Tense::Present, Mood::Indicative, Voice::Active, Person::First, Number::Singular);
/// assert_eq!(result, "λεγω");
///
/// // Conjugate "λυ" (to loose) in Aorist Active Indicative, 1st Person Plural
/// let result = conjugate("ελυ", Tense::Aorist, Mood::Indicative, Voice::Active, Person::First, Number::Plural);
/// assert_eq!(result, "ελυσαμεν");
/// ```
pub fn conjugate(
    stem: &str,
    tense: Tense,
    mood: Mood,
    voice: Voice,
    person: Person,
    number: Number,
) -> String {
    let endings = match (tense, mood, voice) {
        (Tense::Present, Mood::Indicative, Voice::Active) => PRESENT_ACTIVE_IND,
        (Tense::Present, Mood::Imperative, Voice::Active) => PRESENT_ACTIVE_IMP,
        (Tense::Aorist, Mood::Indicative, Voice::Active) => AORIST_ACTIVE_IND,
        (Tense::Aorist, Mood::Imperative, Voice::Active) => AORIST_ACTIVE_IMP,
        _ => return stem.to_string(),
    };

    for (ending, p, n) in endings {
        if *p == person && *n == number {
            return format!("{}{}", stem, ending);
        }
    }

    stem.to_string()
}

/// Get the infinitive form of a verb
///
/// Returns the infinitive form of a verb stem given its tense and voice.
/// Infinitives are used as noun-equivalents or in specific syntactic constructions.
///
/// # Examples
///
/// ```rust
/// use glossa::morphology::{infinitive, Tense, Voice};
///
/// // Present Active Infinitive of "λεγ" (to say)
/// let result = infinitive("λεγ", Tense::Present, Voice::Active);
/// assert_eq!(result, "λεγειν");
///
/// // Aorist Active Infinitive of "λυ" (to loose)
/// let result = infinitive("λυ", Tense::Aorist, Voice::Active);
/// assert_eq!(result, "λυσαι");
/// ```
pub fn infinitive(stem: &str, tense: Tense, voice: Voice) -> String {
    match (tense, voice) {
        (Tense::Present, Voice::Active) => format!("{}{}", stem, PRESENT_INFINITIVE),
        (Tense::Aorist, Voice::Active) => format!("{}{}", stem, AORIST_INFINITIVE),
        _ => stem.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_present_active_indicative() {
        let analysis = analyze_verb("λεγω").unwrap();
        assert_eq!(analysis.tense, Some(Tense::Present));
        assert_eq!(analysis.mood, Some(Mood::Indicative));
        assert_eq!(analysis.person, Some(Person::First));
        assert_eq!(analysis.number, Some(Number::Singular));

        let analysis = analyze_verb("γραφει").unwrap();
        assert_eq!(analysis.person, Some(Person::Third));
        assert_eq!(analysis.number, Some(Number::Singular));
    }

    #[test]
    fn test_present_active_imperative() {
        let analysis = analyze_verb("λεγε").unwrap();
        assert_eq!(analysis.tense, Some(Tense::Present));
        assert_eq!(analysis.mood, Some(Mood::Imperative));
        assert_eq!(analysis.person, Some(Person::Second));
        assert_eq!(analysis.number, Some(Number::Singular));
    }

    #[test]
    fn test_aorist_active() {
        let analysis = analyze_verb("ελυσα").unwrap();
        assert_eq!(analysis.tense, Some(Tense::Aorist));
        assert_eq!(analysis.mood, Some(Mood::Indicative));
        assert_eq!(analysis.person, Some(Person::First));
        // Augment stripped: ελυ → λυ, so lemma is λυω not ελυω
        assert_eq!(analysis.lemma, "λυω");
    }

    #[test]
    fn test_augment_stripping() {
        // Standard sigmatic aorist with ε-augment
        // ελυσα = ε (augment) + λυ (stem) + σα (aorist ending)
        let analysis = analyze_verb("ελυσα").unwrap();
        assert_eq!(analysis.lemma, "λυω");
        assert_eq!(analysis.tense, Some(Tense::Aorist));

        // επαυσα = ε (augment) + παυ (stem) + σα (ending)
        let analysis = analyze_verb("επαυσα").unwrap();
        assert_eq!(analysis.lemma, "παυω");
    }

    #[test]
    fn test_strip_augment_function() {
        assert_eq!(strip_augment("ελυ"), "λυ");
        assert_eq!(strip_augment("εγραψ"), "γραψ");
        assert_eq!(strip_augment("ηγαγ"), "αγαγ"); // η → α restoration
        assert_eq!(strip_augment("λυ"), "λυ"); // no augment

        // Coverage for irregular aorists
        assert_eq!(strip_augment("ηκουσ"), "ακου"); // ἤκουσα -> ἀκούω
        assert_eq!(strip_augment("ωνομασ"), "ονομαζ"); // ὠνόμασα -> ὀνομάζω

        // Coverage for verbs starting with epsilon (should not be stripped)
        assert_eq!(strip_augment("ελπιζ"), "ελπιζ"); // ἐλπίζω
        assert_eq!(strip_augment("εχ"), "εχ"); // ἔχω

        // Coverage for omega augment (ο -> ω)
        // Using a synthetic case or one not in irregular list
        // ὠρίσθην -> ὁρίζω (stem ωρισ -> ορισ)
        assert_eq!(strip_augment("ωρισ"), "ορισ");
    }

    #[test]
    fn test_infinitive() {
        let analysis = analyze_verb("λεγειν").unwrap();
        assert_eq!(analysis.tense, Some(Tense::Present));
        assert_eq!(analysis.mood, Some(Mood::Infinitive));
    }

    #[test]
    fn test_eimi_subjunctive() {
        // ᾖ normalizes to η - 3rd person singular subjunctive of εἰμί
        let analysis = analyze_verb("η").unwrap();
        assert_eq!(analysis.lemma, "ειμι");
        assert_eq!(analysis.mood, Some(Mood::Subjunctive));
        assert_eq!(analysis.person, Some(Person::Third));
        assert_eq!(analysis.number, Some(Number::Singular));
    }

    #[test]
    fn test_conjugate() {
        assert_eq!(
            conjugate(
                "λεγ",
                Tense::Present,
                Mood::Indicative,
                Voice::Active,
                Person::First,
                Number::Singular
            ),
            "λεγω"
        );
        assert_eq!(
            conjugate(
                "λεγ",
                Tense::Present,
                Mood::Imperative,
                Voice::Active,
                Person::Second,
                Number::Singular
            ),
            "λεγε"
        );
    }

    #[test]
    fn test_infinitive_form() {
        assert_eq!(infinitive("λεγ", Tense::Present, Voice::Active), "λεγειν");
        assert_eq!(infinitive("λυ", Tense::Aorist, Voice::Active), "λυσαι");
    }

    #[test]
    fn test_analyze_verb_coverage_forms() {
        // Aorist Active Infinitive
        let analysis = analyze_verb("λυσαι").unwrap();
        assert_eq!(analysis.tense, Some(Tense::Aorist));
        assert_eq!(analysis.mood, Some(Mood::Infinitive));
        assert_eq!(analysis.lemma, "λυω"); // Constructed from stem "λυ" + "ω" (Aorist Infinitive ending is "σαι")

        // Present Active Subjunctive
        let analysis = analyze_verb("λυῃς").unwrap();
        assert_eq!(analysis.mood, Some(Mood::Subjunctive));
        assert_eq!(analysis.lemma, "λυω");

        // Present Active Optative
        let analysis = analyze_verb("λυοιμι").unwrap();
        assert_eq!(analysis.mood, Some(Mood::Optative));
        assert_eq!(analysis.lemma, "λυω");

        // Aorist Passive Optative
        let analysis = analyze_verb("λυθειη").unwrap();
        assert_eq!(analysis.voice, Some(Voice::Passive));
        assert_eq!(analysis.mood, Some(Mood::Optative));
        assert_eq!(analysis.lemma, "λυω"); // Strip "θ" from "λυθ" -> "λυ"
    }

    #[test]
    fn test_constants_sorted() {
        // Enforce that ending constants are sorted by length descending
        // This is required for the zero-allocation match_verb_endings optimization
        let constant_lists = vec![
            ("PRESENT_ACTIVE_IND", PRESENT_ACTIVE_IND),
            ("PRESENT_ACTIVE_IMP", PRESENT_ACTIVE_IMP),
            ("AORIST_ACTIVE_IND", AORIST_ACTIVE_IND),
            ("AORIST_ACTIVE_IMP", AORIST_ACTIVE_IMP),
            ("PRESENT_ACTIVE_SUBJ", PRESENT_ACTIVE_SUBJ),
            ("AORIST_ACTIVE_SUBJ", AORIST_ACTIVE_SUBJ),
            ("PRESENT_ACTIVE_OPT", PRESENT_ACTIVE_OPT),
            ("AORIST_PASSIVE_OPT", AORIST_PASSIVE_OPT),
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
    fn test_analyze_verb_all_discrepancies() {
        // Aorist Passive Optative: λυθειη -> λυω (previously analyze_verb_all might have failed lemma or missed it if not in list)
        let word = "λυθειη";
        let analyses = analyze_verb_all(word);
        let found = analyses
            .iter()
            .find(|a| a.mood == Some(Mood::Optative) && a.voice == Some(Voice::Passive));
        assert!(
            found.is_some(),
            "analyze_verb_all should find Aorist Passive Optative"
        );
        if let Some(analysis) = found {
            assert_eq!(
                analysis.lemma, "λυω",
                "Lemma should be correctly stripped of passive marker"
            );
        }

        // Subjunctive: λυῃς (Present Active Subjunctive) -> λυω (previously missing from analyze_verb_all)
        // Note: analyze_verb_all expects normalized input (accents removed)
        let word = "λυῃς";
        let normalized = normalize_greek(word);
        let analyses = analyze_verb_all(&normalized);

        // Debug output
        if !analyses.iter().any(|a| a.mood == Some(Mood::Subjunctive)) {
            println!("Word: {}", word);
            println!("Normalized: {}", normalized);
            println!("Analyses: {:?}", analyses);
        }

        let found = analyses.iter().find(|a| a.mood == Some(Mood::Subjunctive));
        assert!(found.is_some(), "analyze_verb_all should find Subjunctive");
    }
}
