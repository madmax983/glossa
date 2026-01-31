//! Verb conjugation tables and analysis
//!
//! Implements verb analysis for ΓΛΩΣΣΑ, focusing on:
//! - Present Active Indicative (streaming operations)
//! - Aorist Active Indicative (one-shot operations)
//! - Imperative (commands)

use std::borrow::Cow;

use super::{Mood, MorphAnalysis, Number, PartOfSpeech, Person, Tense, Voice};
use crate::grammar::normalize_greek;

/// Present Active Indicative endings (ω-conjugation)
/// Pattern: λέγω, γράφω, etc.
const PRESENT_ACTIVE_IND: &[(&str, Person, Number)] = &[
    ("ω", Person::First, Number::Singular),
    ("εις", Person::Second, Number::Singular),
    ("ει", Person::Third, Number::Singular),
    ("ομεν", Person::First, Number::Plural),
    ("ετε", Person::Second, Number::Plural),
    ("ουσι", Person::Third, Number::Plural),
    ("ουσιν", Person::Third, Number::Plural), // with movable nu
];

/// Present Active Imperative endings
const PRESENT_ACTIVE_IMP: &[(&str, Person, Number)] = &[
    ("ε", Person::Second, Number::Singular),
    ("ετω", Person::Third, Number::Singular),
    ("ετε", Person::Second, Number::Plural),
    ("οντων", Person::Third, Number::Plural),
];

/// Aorist Active Indicative endings (first/sigmatic aorist)
/// Pattern: ἔλυσα, ἔγραψα, etc.
const AORIST_ACTIVE_IND: &[(&str, Person, Number)] = &[
    ("σα", Person::First, Number::Singular),
    ("σας", Person::Second, Number::Singular),
    ("σε", Person::Third, Number::Singular),
    ("σεν", Person::Third, Number::Singular), // with movable nu
    ("σαμεν", Person::First, Number::Plural),
    ("σατε", Person::Second, Number::Plural),
    ("σαν", Person::Third, Number::Plural),
];

/// Aorist Active Imperative endings
const AORIST_ACTIVE_IMP: &[(&str, Person, Number)] = &[
    ("σον", Person::Second, Number::Singular),
    ("σατω", Person::Third, Number::Singular),
    ("σατε", Person::Second, Number::Plural),
    ("σαντων", Person::Third, Number::Plural),
];

/// Present Active Subjunctive endings (long vowel theme)
/// Pattern: λύω → λύω (identical to present indicative in form, but subjunctive in meaning)
/// The subjunctive has lengthened thematic vowel: ω/η instead of ο/ε
const PRESENT_ACTIVE_SUBJ: &[(&str, Person, Number)] = &[
    ("ω", Person::First, Number::Singular),
    ("ης", Person::Second, Number::Singular), // Note: η + ς (normalized)
    ("η", Person::Third, Number::Singular),   // Long vowel ῃ (normalized)
    ("ωμεν", Person::First, Number::Plural),
    ("ητε", Person::Second, Number::Plural),
    ("ωσι", Person::Third, Number::Plural),
    ("ωσιν", Person::Third, Number::Plural), // with movable nu
];

/// Aorist Active Subjunctive endings
/// Pattern: λύσω (σ + subjunctive endings)
const AORIST_ACTIVE_SUBJ: &[(&str, Person, Number)] = &[
    ("σω", Person::First, Number::Singular),
    ("σης", Person::Second, Number::Singular),
    ("ση", Person::Third, Number::Singular),
    ("σωμεν", Person::First, Number::Plural),
    ("σητε", Person::Second, Number::Plural),
    ("σωσι", Person::Third, Number::Plural),
    ("σωσιν", Person::Third, Number::Plural),
];

/// Present Active Optative endings
/// Pattern: γράφοιμι "I might write"
/// The optative mood expresses possibility, wish, or potential - natural for Option<T>
const PRESENT_ACTIVE_OPT: &[(&str, Person, Number)] = &[
    ("οιμι", Person::First, Number::Singular),
    ("οις", Person::Second, Number::Singular),
    ("οι", Person::Third, Number::Singular),
    ("οιμεν", Person::First, Number::Plural),
    ("οιτε", Person::Second, Number::Plural),
    ("οιεν", Person::Third, Number::Plural),
];

/// Aorist Passive Optative endings
/// Pattern: εὑρεθείη "might be found"
/// Used for values that "might exist" (Option<T> semantics)
const AORIST_PASSIVE_OPT: &[(&str, Person, Number)] = &[
    ("θειην", Person::First, Number::Singular),
    ("θειης", Person::Second, Number::Singular),
    ("θειη", Person::Third, Number::Singular),
    ("θειημεν", Person::First, Number::Plural),
    ("θειητε", Person::Second, Number::Plural),
    ("θειησαν", Person::Third, Number::Plural),
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
fn strip_augment(augmented_stem: &str) -> String {
    // First, check the irregular aorists lookup table
    for (aorist, present) in IRREGULAR_AORISTS {
        if augmented_stem.starts_with(*aorist) {
            let rest = &augmented_stem[aorist.len()..];
            return format!("{}{}", present, rest);
        }
    }

    // Check if this verb naturally starts with ε (don't strip!)
    for prefix in VERBS_STARTING_WITH_EPSILON {
        if augmented_stem.starts_with(*prefix) {
            return augmented_stem.to_string();
        }
    }

    // Simple epsilon augment (most common)
    // After normalization, this is just "ε"
    if let Some(stripped) = augmented_stem.strip_prefix("ε") {
        // Make sure we're not stripping from a vowel-initial stem
        // that received ε → η or ε → ει augment
        if !stripped.is_empty() {
            return stripped.to_string();
        }
    }

    // Vowel lengthening augments (less common, but important)
    // α → η (e.g., ἄγω → ἤγαγον)
    if let Some(rest) = augmented_stem.strip_prefix("η") {
        // Could be α → η augment, restore α
        // But η could also be original (1st decl stem), so this is heuristic
        if !rest.is_empty() {
            return format!("α{}", rest);
        }
    }

    // ο → ω (e.g., ὀνομάζω → ὠνόμασα)
    if let Some(rest) = augmented_stem.strip_prefix("ω")
        && !rest.is_empty()
    {
        return format!("ο{}", rest);
    }

    // ε → η (ε-contract verbs)
    // αι → ῃ, ει → ῃ, οι → ῳ - these are rarer
    // For MVP, we handle the common cases

    // No augment found, return as-is
    augmented_stem.to_string()
}

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

    // Try present active indicative FIRST (most common mood)
    // Note: -ω ending is ambiguous (indicative vs subjunctive), prefer indicative
    if let Some((stem, person, number)) = match_verb_endings(word, PRESENT_ACTIVE_IND) {
        return Some(MorphAnalysis {
            lemma: Cow::Owned(format!("{}ω", stem)),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(number),
            gender: None,
            person: Some(person),
            tense: Some(Tense::Present),
            mood: Some(Mood::Indicative),
            voice: Some(Voice::Active),
            confidence: 0.8,
        });
    }

    // Try present active imperative
    if let Some((stem, person, number)) = match_verb_endings(word, PRESENT_ACTIVE_IMP) {
        return Some(MorphAnalysis {
            lemma: Cow::Owned(format!("{}ω", stem)),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(number),
            gender: None,
            person: Some(person),
            tense: Some(Tense::Present),
            mood: Some(Mood::Imperative),
            voice: Some(Voice::Active),
            confidence: 0.75,
        });
    }

    // Try aorist active indicative
    if let Some((augmented_stem, person, number)) = match_verb_endings(word, AORIST_ACTIVE_IND) {
        // Strip temporal augment to find true stem: ἔλυσα → ελυ → λυ
        let true_stem = strip_augment(&augmented_stem);
        return Some(MorphAnalysis {
            lemma: Cow::Owned(format!("{}ω", true_stem)),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(number),
            gender: None,
            person: Some(person),
            tense: Some(Tense::Aorist),
            mood: Some(Mood::Indicative),
            voice: Some(Voice::Active),
            confidence: 0.85,
        });
    }

    // Try aorist active imperative (no augment in imperative, but keep consistent)
    if let Some((stem, person, number)) = match_verb_endings(word, AORIST_ACTIVE_IMP) {
        return Some(MorphAnalysis {
            lemma: Cow::Owned(format!("{}ω", stem)),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(number),
            gender: None,
            person: Some(person),
            tense: Some(Tense::Aorist),
            mood: Some(Mood::Imperative),
            voice: Some(Voice::Active),
            confidence: 0.75,
        });
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

    // Try aorist passive optative (εὑρεθείη "might be found")
    // Checked before subjunctive because "θειη" overlaps with normalized "η" (subjunctive)
    if let Some((stem, person, number)) = match_verb_endings(word, AORIST_PASSIVE_OPT) {
        // Aorist passive stem typically ends in θ (from -θη-)
        // For θειη ending, the stem before θ is what we want
        let lemma = if stem.ends_with('θ') {
            // Strip the θη passive marker to get base stem, then add -ω for lemma
            let base_stem = stem.trim_end_matches('θ');
            format!("{}ω", base_stem)
        } else {
            format!("{}ω", stem)
        };

        return Some(MorphAnalysis {
            lemma: Cow::Owned(lemma),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(number),
            gender: None,
            person: Some(person),
            tense: Some(Tense::Aorist),
            mood: Some(Mood::Optative),
            voice: Some(Voice::Passive),
            confidence: 0.75,
        });
    }

    // Try present active subjunctive (checked after indicative due to -ω overlap)
    // Only match distinctive subjunctive endings (ῃς, ῃ, ωμεν, ητε, ωσι)
    if let Some((stem, person, number)) = match_verb_endings(word, PRESENT_ACTIVE_SUBJ) {
        return Some(MorphAnalysis {
            lemma: Cow::Owned(format!("{}ω", stem)),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(number),
            gender: None,
            person: Some(person),
            tense: Some(Tense::Present),
            mood: Some(Mood::Subjunctive),
            voice: Some(Voice::Active),
            confidence: 0.70, // Lower confidence due to rarity
        });
    }

    // Try aorist active subjunctive
    if let Some((stem, person, number)) = match_verb_endings(word, AORIST_ACTIVE_SUBJ) {
        return Some(MorphAnalysis {
            lemma: Cow::Owned(format!("{}ω", stem)),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(number),
            gender: None,
            person: Some(person),
            tense: Some(Tense::Aorist),
            mood: Some(Mood::Subjunctive),
            voice: Some(Voice::Active),
            confidence: 0.70,
        });
    }

    // Try present active optative (for Option<T> semantics)
    if let Some((stem, person, number)) = match_verb_endings(word, PRESENT_ACTIVE_OPT) {
        return Some(MorphAnalysis {
            lemma: Cow::Owned(format!("{}ω", stem)),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(number),
            gender: None,
            person: Some(person),
            tense: Some(Tense::Present),
            mood: Some(Mood::Optative),
            voice: Some(Voice::Active),
            confidence: 0.75,
        });
    }

    None
}

/// Match a word against verb endings
fn match_verb_endings(
    word: &str,
    endings: &[(&str, Person, Number)],
) -> Option<(String, Person, Number)> {
    // Sort by ending length (longest first)
    let mut sorted: Vec<_> = endings.iter().collect();
    sorted.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (ending, person, number) in sorted {
        if let Some(stem) = word.strip_suffix(ending)
            && !stem.is_empty()
        {
            return Some((stem.to_string(), *person, *number));
        }
    }
    None
}

/// Match a word against ALL verb endings (for ambiguity resolution)
fn match_verb_endings_all(
    word: &str,
    endings: &[(&str, Person, Number)],
) -> Vec<(String, Person, Number)> {
    let mut matches = Vec::new();

    for (ending, person, number) in endings {
        if let Some(stem) = word.strip_suffix(ending)
            && !stem.is_empty()
        {
            matches.push((stem.to_string(), *person, *number));
        }
    }

    matches
}

/// Analyze a word as a verb, returning ALL possible analyses
///
/// This handles cases where a form could belong to multiple paradigms.
/// For example, "-ε" could be:
/// - 2nd person singular present imperative (λέγε!)
/// - 3rd person singular aorist indicative (ἔλυσε)
pub fn analyze_verb_all(word: &str) -> Vec<MorphAnalysis> {
    let mut analyses = Vec::new();

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

    // Helper struct for conjugation patterns
    struct ConjPattern {
        endings: &'static [(&'static str, Person, Number)],
        tense: Tense,
        mood: Mood,
        voice: Voice,
        has_augment: bool,
        base_confidence: f32,
    }

    let patterns = [
        ConjPattern {
            endings: PRESENT_ACTIVE_IND,
            tense: Tense::Present,
            mood: Mood::Indicative,
            voice: Voice::Active,
            has_augment: false,
            base_confidence: 0.8,
        },
        ConjPattern {
            endings: PRESENT_ACTIVE_IMP,
            tense: Tense::Present,
            mood: Mood::Imperative,
            voice: Voice::Active,
            has_augment: false,
            base_confidence: 0.75,
        },
        ConjPattern {
            endings: AORIST_ACTIVE_IND,
            tense: Tense::Aorist,
            mood: Mood::Indicative,
            voice: Voice::Active,
            has_augment: true,
            base_confidence: 0.85,
        },
        ConjPattern {
            endings: AORIST_ACTIVE_IMP,
            tense: Tense::Aorist,
            mood: Mood::Imperative,
            voice: Voice::Active,
            has_augment: false,
            base_confidence: 0.75,
        },
        ConjPattern {
            endings: PRESENT_ACTIVE_OPT,
            tense: Tense::Present,
            mood: Mood::Optative,
            voice: Voice::Active,
            has_augment: false,
            base_confidence: 0.75,
        },
        ConjPattern {
            endings: AORIST_PASSIVE_OPT,
            tense: Tense::Aorist,
            mood: Mood::Optative,
            voice: Voice::Passive,
            has_augment: false,
            base_confidence: 0.75,
        },
    ];

    for pattern in &patterns {
        let matches = match_verb_endings_all(word, pattern.endings);

        for (stem, person, number) in matches {
            // Handle augment stripping for indicative aorists
            let lemma_stem = if pattern.has_augment {
                strip_augment(&stem)
            } else {
                stem.clone()
            };

            // Calculate confidence
            let ending_len = word.len() - stem.len();
            let length_bonus = (ending_len as f32 - 1.0) * 0.03;
            let confidence = (pattern.base_confidence + length_bonus).min(0.95);

            analyses.push(MorphAnalysis {
                lemma: Cow::Owned(format!("{}ω", lemma_stem)),
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
        }
    }

    // Try infinitives
    if let Some(stem) = word.strip_suffix(PRESENT_INFINITIVE)
        && !stem.is_empty()
    {
        analyses.push(MorphAnalysis {
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

    if let Some(stem) = word.strip_suffix(AORIST_INFINITIVE)
        && !stem.is_empty()
    {
        analyses.push(MorphAnalysis {
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

    // Deduplicate identical analyses
    analyses.sort_by(|a, b| {
        let key_a = (a.tense, a.mood, a.person, a.number, &a.lemma);
        let key_b = (b.tense, b.mood, b.person, b.number, &b.lemma);
        format!("{:?}", key_a).cmp(&format!("{:?}", key_b))
    });
    analyses.dedup_by(|a, b| {
        a.tense == b.tense
            && a.mood == b.mood
            && a.person == b.person
            && a.number == b.number
            && a.lemma == b.lemma
    });

    analyses
}

/// Conjugate a verb stem to a specific form
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
}
