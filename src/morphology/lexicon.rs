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

    // ὠθέω - to push (collection operation)
    m.insert("ωθει", LexiconEntry {
        lemma: "ωθεω".to_string(),
        pos: PartOfSpeech::Verb,
        gender: None,
        meaning: "pushes (collection operation)",
        rust_equiv: Some(".push"),
        case: None,
        number: Some(Number::Singular),
        person: Some(Person::Third),
        tense: Some(Tense::Present),
        mood: Some(Mood::Indicative),
        voice: Some(Voice::Active),
    });

    // ἕλκω - to pull/pop (collection operation, middle voice)
    m.insert("ελκεται", LexiconEntry {
        lemma: "ελκω".to_string(),
        pos: PartOfSpeech::Verb,
        gender: None,
        meaning: "pulls itself (pop, middle voice)",
        rust_equiv: Some(".pop"),
        case: None,
        number: Some(Number::Singular),
        person: Some(Person::Third),
        tense: Some(Tense::Present),
        mood: Some(Mood::Indicative),
        voice: Some(Voice::Middle),
    });

    // μῆκος - length (property noun for collections)
    m.insert("μηκος", LexiconEntry {
        lemma: "μηκος".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Neuter),
        meaning: "length",
        rust_equiv: Some(".len()"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // =========================================================================
    // Ordinal numbers (for array indexing)
    // =========================================================================

    // πρῶτος - first (1st = index 0)
    m.insert("πρωτον", LexiconEntry {
        lemma: "πρωτος".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Neuter),
        meaning: "first",
        rust_equiv: Some("[0]"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    m.insert("πρωτου", LexiconEntry {
        lemma: "πρωτος".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Neuter),
        meaning: "of first",
        rust_equiv: Some("[0]"),
        case: Some(Case::Genitive),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // δεύτερος - second (2nd = index 1)
    m.insert("δευτερον", LexiconEntry {
        lemma: "δευτερος".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Neuter),
        meaning: "second",
        rust_equiv: Some("[1]"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    m.insert("δευτερου", LexiconEntry {
        lemma: "δευτερος".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Neuter),
        meaning: "of second",
        rust_equiv: Some("[1]"),
        case: Some(Case::Genitive),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // τρίτος - third (3rd = index 2)
    m.insert("τριτον", LexiconEntry {
        lemma: "τριτος".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Neuter),
        meaning: "third",
        rust_equiv: Some("[2]"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    m.insert("τριτου", LexiconEntry {
        lemma: "τριτος".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Neuter),
        meaning: "of third",
        rust_equiv: Some("[2]"),
        case: Some(Case::Genitive),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // τέταρτος - fourth (4th = index 3)
    m.insert("τεταρτον", LexiconEntry {
        lemma: "τεταρτος".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Neuter),
        meaning: "fourth",
        rust_equiv: Some("[3]"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // πέμπτος - fifth (5th = index 4)
    m.insert("πεμπτον", LexiconEntry {
        lemma: "πεμπτος".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Neuter),
        meaning: "fifth",
        rust_equiv: Some("[4]"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
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

    // NOTE: ἤ (or) is NOT in the lexicon - handled by boolean_operator()
    // This avoids conflict with ᾖ (subjunctive of εἰμί) which also normalizes to η

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

    // =========================================================================
    // Comparison adjectives (map to operators)
    // =========================================================================

    // μεῖζον - greater (>)
    m.insert("μειζον", LexiconEntry {
        lemma: "μεγας".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Neuter),
        meaning: "greater (comparative)",
        rust_equiv: Some(">"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // ἔλαττον - lesser (<)
    m.insert("ελαττον", LexiconEntry {
        lemma: "ελαττων".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Neuter),
        meaning: "lesser (comparative)",
        rust_equiv: Some("<"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // ἴσον - equal (==)
    m.insert("ισον", LexiconEntry {
        lemma: "ισος".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Neuter),
        meaning: "equal",
        rust_equiv: Some("=="),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // ἄνισον - unequal (!=)
    m.insert("ανισον", LexiconEntry {
        lemma: "ανισος".to_string(),
        pos: PartOfSpeech::Adjective,
        gender: Some(Gender::Neuter),
        meaning: "unequal",
        rust_equiv: Some("!="),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // =========================================================================
    // Boolean particles (map to operators)
    // =========================================================================

    // καί - and (&&)
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

    // NOTE: ἤ (or) handled by boolean_operator(), not lexicon
    // Avoids conflict with ᾖ (subjunctive εἰμί)

    // οὐ/οὐκ/οὐχ - not (!)
    for neg in ["ου", "ουκ", "ουχ"] {
        m.insert(neg, LexiconEntry {
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
    }

    // =========================================================================
    // Arithmetic nouns (map to operators)
    // =========================================================================

    // ἄθροισμα - sum (+)
    m.insert("αθροισμα", LexiconEntry {
        lemma: "αθροισμα".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Neuter),
        meaning: "sum",
        rust_equiv: Some("+"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // διαφορά - difference (-)
    m.insert("διαφορα", LexiconEntry {
        lemma: "διαφορα".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Feminine),
        meaning: "difference",
        rust_equiv: Some("-"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // γινόμενον - product (*)
    m.insert("γινομενον", LexiconEntry {
        lemma: "γινομενον".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Neuter),
        meaning: "product",
        rust_equiv: Some("*"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // μέρος - quotient/part (/)
    m.insert("μερος", LexiconEntry {
        lemma: "μερος".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Neuter),
        meaning: "quotient, part",
        rust_equiv: Some("/"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // ὑπόλοιπον - remainder (%)
    m.insert("υπολοιπον", LexiconEntry {
        lemma: "υπολοιπον".to_string(),
        pos: PartOfSpeech::Noun,
        gender: Some(Gender::Neuter),
        meaning: "remainder",
        rust_equiv: Some("%"),
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

    // =========================================================================
    // Greek numerals as words
    // =========================================================================

    for (word, _value) in [
        ("μηδεν", 0),  // zero (nominative/accusative neuter)
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

    // Add declined forms of zero (μηδέν)
    m.insert("μηδενος", LexiconEntry {
        lemma: "μηδεν".to_string(),
        pos: PartOfSpeech::Numeral,
        gender: Some(Gender::Neuter),
        meaning: "of zero, of nothing",
        rust_equiv: None,
        case: Some(Case::Genitive),
        number: Some(Number::Singular),
        person: None,
        tense: None,
        mood: None,
        voice: None,
    });

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
/// Includes all case forms (nominative, genitive, dative, accusative)
pub fn numeral_value(normalized_word: &str) -> Option<i64> {
    match normalized_word {
        // 0 - μηδέν (nothing/zero)
        "μηδεν" | "μηδενος" => Some(0),
        // 1 - εἷς, μία, ἕν
        "εν" | "ενα" | "ενος" | "μια" | "μιας" => Some(1),
        // 2 - δύο (indeclinable)
        "δυο" | "δυοιν" => Some(2),
        // 3 - τρεῖς, τρία
        "τρια" | "τρεις" | "τριων" | "τρισι" | "τρισιν" => Some(3),
        // 4 - τέτταρες, τέτταρα
        "τεσσαρα" | "τεσσαρες" | "τεσσαρων" | "τεσσαρσι" | "τεσσαρσιν" => Some(4),
        // 5 - πέντε (indeclinable)
        "πεντε" => Some(5),
        // 6 - ἕξ (indeclinable)
        "εξ" => Some(6),
        // 7 - ἑπτά (indeclinable)
        "επτα" => Some(7),
        // 8 - ὀκτώ (indeclinable)
        "οκτω" => Some(8),
        // 9 - ἐννέα (indeclinable)
        "εννεα" => Some(9),
        // 10 - δέκα (indeclinable)
        "δεκα" => Some(10),
        // 100 - ἑκατόν (indeclinable)
        "εκατον" => Some(100),
        // 1000 - χίλιοι/χίλια
        "χιλια" | "χιλιοι" | "χιλιων" => Some(1000),
        _ => None,
    }
}

/// Binary operator type for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Boolean
    And,
    Or,
}

impl BinaryOp {
    /// Get the Rust operator string
    pub fn to_rust(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
    }
}

/// Unary operator type for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,  // οὐ/οὐκ
    Neg,  // arithmetic negation
}

impl UnaryOp {
    /// Get the Rust operator string
    pub fn to_rust(&self) -> &'static str {
        match self {
            UnaryOp::Not => "!",
            UnaryOp::Neg => "-",
        }
    }
}

/// Check if a word is a comparison adjective and return the operator
pub fn comparison_operator(normalized_word: &str) -> Option<BinaryOp> {
    match normalized_word {
        "μειζον" => Some(BinaryOp::Gt),
        "ελαττον" => Some(BinaryOp::Lt),
        "ισον" => Some(BinaryOp::Eq),
        "ανισον" => Some(BinaryOp::Ne),
        _ => None,
    }
}

/// Check if a word is a boolean conjunction and return the operator
pub fn boolean_operator(normalized_word: &str) -> Option<BinaryOp> {
    match normalized_word {
        "και" => Some(BinaryOp::And),
        "η" => Some(BinaryOp::Or),
        _ => None,
    }
}

/// Check if a word is a negation particle
pub fn is_negation(normalized_word: &str) -> bool {
    matches!(normalized_word, "ου" | "ουκ" | "ουχ")
}

/// Check if a word is an arithmetic noun and return the operator
pub fn arithmetic_operator(normalized_word: &str) -> Option<BinaryOp> {
    match normalized_word {
        "αθροισμα" => Some(BinaryOp::Add),
        "διαφορα" => Some(BinaryOp::Sub),
        "γινομενον" => Some(BinaryOp::Mul),
        "μερος" => Some(BinaryOp::Div),
        "υπολοιπον" => Some(BinaryOp::Mod),
        _ => None,
    }
}

/// Check if a word maps to any binary operator
pub fn is_operator_word(normalized_word: &str) -> bool {
    comparison_operator(normalized_word).is_some()
        || boolean_operator(normalized_word).is_some()
        || arithmetic_operator(normalized_word).is_some()
}

/// Get the binary operator for any operator word
pub fn get_binary_operator(normalized_word: &str) -> Option<BinaryOp> {
    comparison_operator(normalized_word)
        .or_else(|| boolean_operator(normalized_word))
        .or_else(|| arithmetic_operator(normalized_word))
}

// =============================================================================
// Control Flow Particles (Phase 2)
// =============================================================================

/// Check if a word is a conditional particle (if)
pub fn is_conditional_particle(normalized_word: &str) -> bool {
    matches!(normalized_word, "ει" | "εαν" | "ην" | "αν")
}

/// Check if a sequence is the else pattern (εἰ δὲ μή)
pub fn is_else_pattern(normalized_phrase: &str) -> bool {
    normalized_phrase == "ει δε μη"
}

/// Check if a word is a loop particle
pub fn is_loop_particle(normalized_word: &str) -> bool {
    matches!(normalized_word, "εως" | "δια")
}

/// Check if a word is a range particle
pub fn is_range_particle(normalized_word: &str) -> bool {
    matches!(normalized_word, "απο" | "μεχρι" | "εως")
}

/// Check if a word is the break verb (παῦε)
pub fn is_break_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "παυε" | "παυω")
}

/// Check if a word is the continue verb (συνέχιζε)
pub fn is_continue_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "συνεχιζε" | "συνεχιζω")
}

/// Check if a word is the match particle (κατά)
pub fn is_match_particle(normalized_word: &str) -> bool {
    normalized_word == "κατα"
}

// =============================================================================
// Collection Operations (Phase 3)
// =============================================================================

/// Check if a word is a push verb (ὠθεῖ/ὠθέω)
pub fn is_push_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "ωθει" | "ωθεω" | "ωθω")
}

/// Check if a word is a pop verb (ἕλκεται/ἕλκομαι - middle voice)
pub fn is_pop_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "ελκεται" | "ελκομαι" | "ελκω")
}

/// Check if a word is the length property (μῆκος)
pub fn is_length_property(normalized_word: &str) -> bool {
    normalized_word == "μηκος"
}

/// Check if a word is an ordinal adjective
pub fn is_ordinal(normalized_word: &str) -> bool {
    matches!(
        normalized_word,
        "πρωτον" | "πρωτου" |
        "δευτερον" | "δευτερου" |
        "τριτον" | "τριτου" |
        "τεταρτον" | "τεταρτου" |
        "πεμπτον" | "πεμπτου"
    )
}

/// Convert an ordinal word to a zero-based array index
/// Returns None if the word is not a recognized ordinal
pub fn ordinal_to_index(normalized_word: &str) -> Option<i64> {
    match normalized_word {
        "πρωτον" | "πρωτου" => Some(0),        // first = 0
        "δευτερον" | "δευτερου" => Some(1),    // second = 1
        "τριτον" | "τριτου" => Some(2),        // third = 2
        "τεταρτον" | "τεταρτου" => Some(3),    // fourth = 3
        "πεμπτον" | "πεμπτου" => Some(4),      // fifth = 4
        _ => None,
    }
}

/// Check if a word is a necessity particle (ἀνάγκη - necessarily)
/// This is the Aristotelian marker for logical consequence
pub fn is_necessity_particle(normalized_word: &str) -> bool {
    matches!(normalized_word, "αναγκη" | "αναγκαιον")
}

/// Check if a word is an obligation particle (δεῖ - must/ought)
/// Less absolute than ἀνάγκη, used for practical necessity
pub fn is_obligation_particle(normalized_word: &str) -> bool {
    matches!(normalized_word, "δει" | "δεον" | "χρη")
}

/// Check if a word introduces a consequence (ἀνάγκη or δεῖ)
pub fn is_consequence_marker(normalized_word: &str) -> bool {
    is_necessity_particle(normalized_word) || is_obligation_particle(normalized_word)
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

    #[test]
    fn test_comparison_operators() {
        assert_eq!(comparison_operator("μειζον"), Some(BinaryOp::Gt));
        assert_eq!(comparison_operator("ελαττον"), Some(BinaryOp::Lt));
        assert_eq!(comparison_operator("ισον"), Some(BinaryOp::Eq));
        assert_eq!(comparison_operator("ανισον"), Some(BinaryOp::Ne));
        assert_eq!(comparison_operator("foo"), None);
    }

    #[test]
    fn test_boolean_operators() {
        assert_eq!(boolean_operator("και"), Some(BinaryOp::And));
        assert_eq!(boolean_operator("η"), Some(BinaryOp::Or));
        assert!(is_negation("ου"));
        assert!(is_negation("ουκ"));
        assert!(is_negation("ουχ"));
    }

    #[test]
    fn test_arithmetic_operators() {
        assert_eq!(arithmetic_operator("αθροισμα"), Some(BinaryOp::Add));
        assert_eq!(arithmetic_operator("διαφορα"), Some(BinaryOp::Sub));
        assert_eq!(arithmetic_operator("γινομενον"), Some(BinaryOp::Mul));
        assert_eq!(arithmetic_operator("μερος"), Some(BinaryOp::Div));
        assert_eq!(arithmetic_operator("υπολοιπον"), Some(BinaryOp::Mod));
    }

    #[test]
    fn test_operator_to_rust() {
        assert_eq!(BinaryOp::Add.to_rust(), "+");
        assert_eq!(BinaryOp::Gt.to_rust(), ">");
        assert_eq!(BinaryOp::And.to_rust(), "&&");
        assert_eq!(UnaryOp::Not.to_rust(), "!");
    }

    #[test]
    fn test_operator_lexicon_entries() {
        // Comparison adjectives
        let entry = lookup("μειζον").unwrap();
        assert_eq!(entry.rust_equiv, Some(">"));
        assert_eq!(entry.pos, PartOfSpeech::Adjective);

        // Boolean particles
        let entry = lookup("και").unwrap();
        assert_eq!(entry.rust_equiv, Some("&&"));
        assert_eq!(entry.pos, PartOfSpeech::Conjunction);

        // Arithmetic nouns
        let entry = lookup("αθροισμα").unwrap();
        assert_eq!(entry.rust_equiv, Some("+"));
        assert_eq!(entry.pos, PartOfSpeech::Noun);
    }
}
