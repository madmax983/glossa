//! Built-in vocabulary and lexicon lookup
//!
//! Contains known words, irregular forms, and built-in functions
//! for ΓΛΩΣΣΑ.

use super::{
    Case, Number, Gender, Person, Tense, Mood, Voice,
    MorphAnalysis, PartOfSpeech,
};
use rustc_hash::FxHashMap;
use std::sync::LazyLock;

/// A lexicon entry with full morphological information
#[derive(Debug, Clone)]
pub struct LexiconEntry {
    /// The dictionary form (lemma)
    pub lemma: String,
    /// Part of speech
    pub pos: PartOfSpeech,
    /// Gender (for nouns/adjectives)
    pub gender: Option<Gender>,
    /// Semantic meaning in the language
    pub meaning: &'static str,
    /// Rust equivalent (for code generation)
    pub rust_equiv: Option<&'static str>,
    /// Grammatical features for this specific form
    pub case: Option<Case>,
    pub number: Option<Number>,
    pub person: Option<Person>,
    pub tense: Option<Tense>,
    pub mood: Option<Mood>,
    pub voice: Option<Voice>,
}

impl LexiconEntry {
    pub fn to_analysis(&self) -> MorphAnalysis {
        MorphAnalysis {
            lemma: self.lemma.clone(),
            part_of_speech: self.pos,
            case: self.case,
            number: self.number,
            gender: self.gender,
            person: self.person,
            tense: self.tense,
            mood: self.mood,
            voice: self.voice,
            confidence: 1.0, // Lexicon entries are definitive
        }
    }
}

/// The built-in lexicon
static LEXICON: LazyLock<FxHashMap<&'static str, LexiconEntry>> = LazyLock::new(|| {
    let mut m = FxHashMap::default();

    // =========================================================================
    // Built-in verbs
    // =========================================================================

    // λέγω - to say/print
    m.insert("λεγω", LexiconEntry {
        lemma: "λεγω".to_string(),
        pos: PartOfSpeech::Verb,
        gender: None,
        meaning: "say, speak, print",
        rust_equiv: Some("println!"),
        case: None,
        number: Some(Number::Singular),
        person: Some(Person::First),
        tense: Some(Tense::Present),
        mood: Some(Mood::Indicative),
        voice: Some(Voice::Active),
    });

    m.insert("λεγε", LexiconEntry {
        lemma: "λεγω".to_string(),
        pos: PartOfSpeech::Verb,
        gender: None,
        meaning: "say! (imperative)",
        rust_equiv: Some("println!"),
        case: None,
        number: Some(Number::Singular),
        person: Some(Person::Second),
        tense: Some(Tense::Present),
        mood: Some(Mood::Imperative),
        voice: Some(Voice::Active),
    });

    // ἔστω - let it be (variable binding)
    m.insert("εστω", LexiconEntry {
        lemma: "ειμι".to_string(),
        pos: PartOfSpeech::Verb,
        gender: None,
        meaning: "let it be (binding)",
        rust_equiv: Some("let"),
        case: None,
        number: Some(Number::Singular),
        person: Some(Person::Third),
        tense: Some(Tense::Present),
        mood: Some(Mood::Imperative),
        voice: Some(Voice::Active),
    });

    // γράφω - to write
    m.insert("γραφω", LexiconEntry {
        lemma: "γραφω".to_string(),
        pos: PartOfSpeech::Verb,
        gender: None,
        meaning: "write",
        rust_equiv: Some("write!"),
        case: None,
        number: Some(Number::Singular),
        person: Some(Person::First),
        tense: Some(Tense::Present),
        mood: Some(Mood::Indicative),
        voice: Some(Voice::Active),
    });

    m.insert("γραφε", LexiconEntry {
        lemma: "γραφω".to_string(),
        pos: PartOfSpeech::Verb,
        gender: None,
        meaning: "write! (imperative)",
        rust_equiv: Some("print!"),
        case: None,
        number: Some(Number::Singular),
        person: Some(Person::Second),
        tense: Some(Tense::Present),
        mood: Some(Mood::Imperative),
        voice: Some(Voice::Active),
    });

    // =========================================================================
    // Built-in type nouns
    // =========================================================================

    // ἀριθμός - number (i64)
    m.insert("αριθμος", LexiconEntry {
        lemma: "αριθμος".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Masculine),
        meaning: "number",
        rust_equiv: Some("i64"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // ὄνομα - name/string
    m.insert("ονομα", LexiconEntry {
        lemma: "ονομα".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Neuter),
        meaning: "name, string",
        rust_equiv: Some("String"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    m.insert("ονοματος", LexiconEntry {
        lemma: "ονομα".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Neuter),
        meaning: "of a name/string",
        rust_equiv: Some("&String"),
        case: Some(Case::Genitive),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // λίστη - list
    m.insert("λιστη", LexiconEntry {
        lemma: "λιστη".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Feminine),
        meaning: "list",
        rust_equiv: Some("Vec"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // κενόν - unit/void type
    m.insert("κενον", LexiconEntry {
        lemma: "κενον".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Neuter),
        meaning: "empty, void",
        rust_equiv: Some("()"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // χαρακτήρ - character type
    m.insert("χαρακτηρ", LexiconEntry {
        lemma: "χαρακτηρ".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Masculine),
        meaning: "character",
        rust_equiv: Some("char"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // βάσις - byte type
    m.insert("βασις", LexiconEntry {
        lemma: "βασις".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Feminine),
        meaning: "base, byte",
        rust_equiv: Some("u8"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // δεσμός - tuple/bond
    m.insert("δεσμος", LexiconEntry {
        lemma: "δεσμος".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Masculine),
        meaning: "bond, tuple",
        rust_equiv: Some("tuple"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // χάρτης - map/dictionary
    m.insert("χαρτης", LexiconEntry {
        lemma: "χαρτης".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Masculine),
        meaning: "map, chart",
        rust_equiv: Some("HashMap"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // σύνολον - set
    m.insert("συνολον", LexiconEntry {
        lemma: "συνολον".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Neuter),
        meaning: "collection, set",
        rust_equiv: Some("HashSet"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // =========================================================================
    // More verbs for common operations
    // =========================================================================

    // ποιέω - to make/do (general action)
    m.insert("ποιεω", LexiconEntry {
        lemma: "ποιεω".to_string(),
        pos: PartOfSpeech::Verb,
        gender: None,
        meaning: "make, do",
        rust_equiv: None,
        case: None,
        number: Some(Number::Singular),
        person: Some(Person::First),
        tense: Some(Tense::Present),
        mood: Some(Mood::Indicative),
        voice: Some(Voice::Active),
    });

    m.insert("ποιει", LexiconEntry {
        lemma: "ποιεω".to_string(),
        pos: PartOfSpeech::Verb,
        gender: None,
        meaning: "makes, does",
        rust_equiv: None,
        case: None,
        number: Some(Number::Singular),
        person: Some(Person::Third),
        tense: Some(Tense::Present),
        mood: Some(Mood::Indicative),
        voice: Some(Voice::Active),
    });

    // λαμβάνω - to take/receive (input)
    m.insert("λαμβανω", LexiconEntry {
        lemma: "λαμβανω".to_string(),
        pos: PartOfSpeech::Verb,
        gender: None,
        meaning: "take, receive, input",
        rust_equiv: Some("read_line"),
        case: None,
        number: Some(Number::Singular),
        person: Some(Person::First),
        tense: Some(Tense::Present),
        mood: Some(Mood::Indicative),
        voice: Some(Voice::Active),
    });

    m.insert("λαβε", LexiconEntry {
        lemma: "λαμβανω".to_string(),
        pos: PartOfSpeech::Verb,
        gender: None,
        meaning: "take! receive! (imperative)",
        rust_equiv: Some("read_line"),
        case: None,
        number: Some(Number::Singular),
        person: Some(Person::Second),
        tense: Some(Tense::Aorist),
        mood: Some(Mood::Imperative),
        voice: Some(Voice::Active),
    });

    // δίδωμι - to give (output/return)
    m.insert("διδωμι", LexiconEntry {
        lemma: "διδωμι".to_string(),
        pos: PartOfSpeech::Verb,
        gender: None,
        meaning: "give, return",
        rust_equiv: Some("return"),
        case: None,
        number: Some(Number::Singular),
        person: Some(Person::First),
        tense: Some(Tense::Present),
        mood: Some(Mood::Indicative),
        voice: Some(Voice::Active),
    });

    m.insert("δος", LexiconEntry {
        lemma: "διδωμι".to_string(),
        pos: PartOfSpeech::Verb,
        gender: None,
        meaning: "give! (imperative)",
        rust_equiv: Some("return"),
        case: None,
        number: Some(Number::Singular),
        person: Some(Person::Second),
        tense: Some(Tense::Aorist),
        mood: Some(Mood::Imperative),
        voice: Some(Voice::Active),
    });

    // εἰ / ἐάν - conditional
    m.insert("ει", LexiconEntry {
        lemma: "ει".to_string(),
        pos: PartOfSpeech::Particle,
        gender: None,
        meaning: "if",
        rust_equiv: Some("if"),
        case: None,
        number: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    m.insert("εαν", LexiconEntry {
        lemma: "εαν".to_string(),
        pos: PartOfSpeech::Particle,
        gender: None,
        meaning: "if (with subjunctive)",
        rust_equiv: Some("if"),
        case: None,
        number: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // ἄλλως - else
    m.insert("αλλως", LexiconEntry {
        lemma: "αλλως".to_string(),
        pos: PartOfSpeech::Particle,
        gender: None,
        meaning: "otherwise, else",
        rust_equiv: Some("else"),
        case: None,
        number: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // ἕως - while/until
    m.insert("εως", LexiconEntry {
        lemma: "εως".to_string(),
        pos: PartOfSpeech::Particle,
        gender: None,
        meaning: "while, until",
        rust_equiv: Some("while"),
        case: None,
        number: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // διά - for (iteration)
    m.insert("δια", LexiconEntry {
        lemma: "δια".to_string(),
        pos: PartOfSpeech::Preposition,
        gender: None,
        meaning: "through, for each",
        rust_equiv: Some("for"),
        case: None,
        number: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // ἐν - in (membership)
    m.insert("εν", LexiconEntry {
        lemma: "εν".to_string(),
        pos: PartOfSpeech::Preposition,
        gender: None,
        meaning: "in",
        rust_equiv: Some("in"),
        case: None,
        number: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // καί - and
    m.insert("και", LexiconEntry {
        lemma: "και".to_string(),
        pos: PartOfSpeech::Conjunction,
        gender: None,
        meaning: "and",
        rust_equiv: Some("&&"),
        case: None,
        number: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // ἤ - or
    m.insert("η", LexiconEntry {
        lemma: "η".to_string(),
        pos: PartOfSpeech::Conjunction,
        gender: None,
        meaning: "or",
        rust_equiv: Some("||"),
        case: None,
        number: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // οὐ/οὐκ - not
    m.insert("ου", LexiconEntry {
        lemma: "ου".to_string(),
        pos: PartOfSpeech::Particle,
        gender: None,
        meaning: "not",
        rust_equiv: Some("!"),
        case: None,
        number: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    m.insert("ουκ", LexiconEntry {
        lemma: "ου".to_string(),
        pos: PartOfSpeech::Particle,
        gender: None,
        meaning: "not (before vowels)",
        rust_equiv: Some("!"),
        case: None,
        number: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // =========================================================================
    // Comparison operators
    // =========================================================================

    // ἴσος - equal
    m.insert("ισος", LexiconEntry {
        lemma: "ισος".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Masculine),
        meaning: "equal",
        rust_equiv: Some("=="),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // μείζων - greater
    m.insert("μειζων", LexiconEntry {
        lemma: "μεγας".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Masculine),
        meaning: "greater",
        rust_equiv: Some(">"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // ἐλάσσων - lesser
    m.insert("ελασσων", LexiconEntry {
        lemma: "μικρος".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Masculine),
        meaning: "lesser, smaller",
        rust_equiv: Some("<"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // =========================================================================
    // Boolean literals
    // =========================================================================

    m.insert("αληθες", LexiconEntry {
        lemma: "αληθης".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Neuter),
        meaning: "true",
        rust_equiv: Some("true"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    m.insert("ψευδος", LexiconEntry {
        lemma: "ψευδος".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Neuter),
        meaning: "false, lie",
        rust_equiv: Some("false"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // =========================================================================
    // Common words
    // =========================================================================

    // χρήστης - user
    m.insert("χρηστης", LexiconEntry {
        lemma: "χρηστης".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Masculine),
        meaning: "user",
        rust_equiv: Some("user"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // Greek numerals as words
    for (word, _value) in [
        ("εν", 1), ("ενα", 1),
        ("δυο", 2),
        ("τρια", 3), ("τρεις", 3),
        ("τεσσαρα", 4), ("τεσσαρες", 4),
        ("πεντε", 5),
        ("εξ", 6),
        ("επτα", 7),
        ("οκτω", 8),
        ("εννεα", 9),
        ("δεκα", 10),
        ("εκατον", 100),
        ("χιλια", 1000),
    ] {
        m.insert(word, LexiconEntry {
            lemma: word.to_string(),
            pos: PartOfSpeech::Numeral,
            gender: None,
            meaning: "numeral",
            rust_equiv: None,
            case: None,
            number: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
        });
    }

    m
});

/// Look up a word in the lexicon
pub fn lookup(normalized_word: &str) -> Option<&'static LexiconEntry> {
    LEXICON.get(normalized_word)
}

/// Check if a word is a known verb
pub fn is_verb(normalized_word: &str) -> bool {
    lookup(normalized_word)
        .map(|e| e.pos == PartOfSpeech::Verb)
        .unwrap_or(false)
}

/// Check if a word is a binding verb (ἔστω / εἰμί)
/// This includes the conjugated form (εστω) and the lemma (ειμι)
pub fn is_binding_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "εστω" | "ειμι" | "εστι" | "εισι" | "ειναι")
}

/// Check if a word is a print verb (λέγε, γράφε)
pub fn is_print_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "λεγε" | "γραφε" | "λεγω" | "γραφω")
}

/// Get the numeric value of a Greek numeral word
pub fn numeral_value(normalized_word: &str) -> Option<i64> {
    match normalized_word {
        "εν" | "ενα" => Some(1),
        "δυο" => Some(2),
        "τρια" | "τρεις" => Some(3),
        "τεσσαρα" | "τεσσαρες" => Some(4),
        "πεντε" => Some(5),
        "εξ" => Some(6),
        "επτα" => Some(7),
        "οκτω" => Some(8),
        "εννεα" => Some(9),
        "δεκα" => Some(10),
        "εκατον" => Some(100),
        "χιλια" => Some(1000),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_verb() {
        let entry = lookup("λεγε").unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Verb);
        assert_eq!(entry.mood, Some(Mood::Imperative));
    }

    #[test]
    fn test_lookup_binding() {
        let entry = lookup("εστω").unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Verb);
        assert!(is_binding_verb("εστω"));
    }

    #[test]
    fn test_lookup_type() {
        let entry = lookup("αριθμος").unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Noun);
        assert_eq!(entry.rust_equiv, Some("i64"));
    }

    #[test]
    fn test_is_print_verb() {
        assert!(is_print_verb("λεγε"));
        assert!(is_print_verb("γραφε"));
        assert!(!is_print_verb("εστω"));
    }

    #[test]
    fn test_numeral_value() {
        assert_eq!(numeral_value("πεντε"), Some(5));
        assert_eq!(numeral_value("δεκα"), Some(10));
        assert_eq!(numeral_value("foo"), None);
    }

    #[test]
    fn test_boolean_lookup() {
        let entry = lookup("αληθες").unwrap();
        assert_eq!(entry.rust_equiv, Some("true"));

        let entry = lookup("ψευδος").unwrap();
        assert_eq!(entry.rust_equiv, Some("false"));
    }
}
