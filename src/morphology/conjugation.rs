//! Verb conjugation tables and analysis
//!
//! Implements verb analysis for ΓΛΩΣΣΑ, focusing on:
//! - Present Active Indicative (streaming operations)
//! - Aorist Active Indicative (one-shot operations)
//! - Imperative (commands)

use super::{Person, Number, Tense, Mood, Voice, MorphAnalysis, PartOfSpeech};

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

/// Present Active Infinitive ending
const PRESENT_INFINITIVE: &str = "ειν";

/// Aorist Active Infinitive ending
const AORIST_INFINITIVE: &str = "σαι";

/// Try to analyze a word as a verb
pub fn analyze_verb(word: &str) -> Option<MorphAnalysis> {
    // Try present active indicative
    if let Some((stem, person, number)) = match_verb_endings(word, PRESENT_ACTIVE_IND) {
        return Some(MorphAnalysis {
            lemma: format!("{}ω", stem),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(number),
            gender: None,
            person: Some(person),
            tense: Some(Tense::Present),
            mood: Some(Mood::Indicative),
            voice: Some(Voice::Active),
        });
    }

    // Try present active imperative
    if let Some((stem, person, number)) = match_verb_endings(word, PRESENT_ACTIVE_IMP) {
        return Some(MorphAnalysis {
            lemma: format!("{}ω", stem),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(number),
            gender: None,
            person: Some(person),
            tense: Some(Tense::Present),
            mood: Some(Mood::Imperative),
            voice: Some(Voice::Active),
        });
    }

    // Try aorist active indicative
    if let Some((stem, person, number)) = match_verb_endings(word, AORIST_ACTIVE_IND) {
        return Some(MorphAnalysis {
            lemma: format!("{}ω", stem),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(number),
            gender: None,
            person: Some(person),
            tense: Some(Tense::Aorist),
            mood: Some(Mood::Indicative),
            voice: Some(Voice::Active),
        });
    }

    // Try aorist active imperative
    if let Some((stem, person, number)) = match_verb_endings(word, AORIST_ACTIVE_IMP) {
        return Some(MorphAnalysis {
            lemma: format!("{}ω", stem),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(number),
            gender: None,
            person: Some(person),
            tense: Some(Tense::Aorist),
            mood: Some(Mood::Imperative),
            voice: Some(Voice::Active),
        });
    }

    // Try present infinitive
    if word.ends_with(PRESENT_INFINITIVE) {
        let stem = &word[..word.len() - PRESENT_INFINITIVE.len()];
        if !stem.is_empty() {
            return Some(MorphAnalysis {
                lemma: format!("{}ω", stem),
                part_of_speech: PartOfSpeech::Verb,
                case: None,
                number: None,
                gender: None,
                person: None,
                tense: Some(Tense::Present),
                mood: Some(Mood::Infinitive),
                voice: Some(Voice::Active),
            });
        }
    }

    // Try aorist infinitive
    if word.ends_with(AORIST_INFINITIVE) {
        let stem = &word[..word.len() - AORIST_INFINITIVE.len()];
        if !stem.is_empty() {
            return Some(MorphAnalysis {
                lemma: format!("{}ω", stem),
                part_of_speech: PartOfSpeech::Verb,
                case: None,
                number: None,
                gender: None,
                person: None,
                tense: Some(Tense::Aorist),
                mood: Some(Mood::Infinitive),
                voice: Some(Voice::Active),
            });
        }
    }

    None
}

/// Match a word against verb endings
fn match_verb_endings(word: &str, endings: &[(&str, Person, Number)]) -> Option<(String, Person, Number)> {
    // Sort by ending length (longest first)
    let mut sorted: Vec<_> = endings.iter().collect();
    sorted.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (ending, person, number) in sorted {
        if word.ends_with(ending) {
            let stem = &word[..word.len() - ending.len()];
            if !stem.is_empty() {
                return Some((stem.to_string(), *person, *number));
            }
        }
    }
    None
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
    }

    #[test]
    fn test_infinitive() {
        let analysis = analyze_verb("λεγειν").unwrap();
        assert_eq!(analysis.tense, Some(Tense::Present));
        assert_eq!(analysis.mood, Some(Mood::Infinitive));
    }

    #[test]
    fn test_conjugate() {
        assert_eq!(
            conjugate("λεγ", Tense::Present, Mood::Indicative, Voice::Active, Person::First, Number::Singular),
            "λεγω"
        );
        assert_eq!(
            conjugate("λεγ", Tense::Present, Mood::Imperative, Voice::Active, Person::Second, Number::Singular),
            "λεγε"
        );
    }

    #[test]
    fn test_infinitive_form() {
        assert_eq!(infinitive("λεγ", Tense::Present, Voice::Active), "λεγειν");
        assert_eq!(infinitive("λυ", Tense::Aorist, Voice::Active), "λυσαι");
    }
}
