use crate::morphology::models::{Case, Gender, Mood, Number, PartOfSpeech, Person, Tense, Voice};
use std::borrow::Cow;
use crate::morphology::models::MorphAnalysis;

#[derive(Debug, Clone, Copy)]
pub struct LexiconEntry {
    /// The dictionary form (lemma)
    ///
    /// Example: "λεγω" for the verb "to say".
    pub lemma: &'static str,
    /// Part of speech (Verb, Noun, etc.)
    pub pos: PartOfSpeech,
    /// Gender (only for Nouns, Adjectives, Pronouns, Articles)
    pub gender: Option<Gender>,
    /// Semantic meaning (English description)
    pub meaning: &'static str,
    /// Rust equivalent string (for codegen)
    ///
    /// If `Some`, this string is used directly in the generated Rust code.
    /// Example: `Some("println!")` for `λέγω`.
    pub rust_equiv: Option<&'static str>,
    /// Case (for nominals)
    pub case: Option<Case>,
    /// Number (Singular/Plural)
    pub number: Option<Number>,
    /// Person (First/Second/Third)
    pub person: Option<Person>,
    /// Tense (Present, Aorist, etc.)
    pub tense: Option<Tense>,
    /// Mood (Indicative, Imperative, etc.)
    pub mood: Option<Mood>,
    /// Voice (Active, Middle, Passive)
    pub voice: Option<Voice>,
}

impl LexiconEntry {
    /// Converts a static lexicon entry into a dynamically usable MorphAnalysis.
    ///
    /// This is necessary because the Lexicon is static (and optimized to use `&'static str`),
    /// but during parsing and ambiguity resolution we may need to dynamically adjust
    /// or duplicate these analyses (e.g. creating owned strings for stems we construct).
    ///
    /// The resulting `MorphAnalysis` always has a confidence of `1.0` since lexicon entries
    /// are the definitive source of truth for irregular and core vocabulary.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::morphology::{lookup, Case, PartOfSpeech};
    ///
    /// let entry = lookup("αριθμος").expect("Word not found");
    /// let analysis = entry.to_analysis();
    ///
    /// assert_eq!(analysis.part_of_speech, PartOfSpeech::Noun);
    /// assert_eq!(analysis.case, Some(Case::Nominative));
    /// assert_eq!(analysis.confidence, 1.0);
    /// ```
    pub fn to_analysis(&self) -> MorphAnalysis {
        MorphAnalysis {
            lemma: Cow::Borrowed(self.lemma),
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

/// Binary operator type for code generation
///
/// In ΓΛΩΣΣΑ, operators are frequently expressed as descriptive adjectives or nouns
/// (e.g. `μείζον` for "greater than", `ἄθροισμα` for "sum"). These concepts are mapped
/// to fundamental binary operations during the assembly phase, establishing relations
/// between two distinct expressions.
///
/// ## Examples
///
/// ```rust
/// use glossa::morphology::{BinaryOp, comparison_operator};
///
/// // The Greek word for "equal" (ἴσον) translates directly to the Eq binary operator
/// let op = comparison_operator("ισον").unwrap();
/// assert_eq!(op, BinaryOp::Eq);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    /// Synthesizes two values into their combined magnitude (`ἄθροισμα`).
    Add,
    /// Evaluates the quantitative difference between two values (`διαφορά`).
    Sub,
    /// Expands the magnitude of a value by the factor of another (`γινόμενον`).
    Mul,
    /// Partitions a value into equal segments (`μέρος`).
    Div,
    /// Isolates the remainder left over from a division (`ὑπόλοιπον`).
    Mod,
    // Comparison
    /// Tests if the identities of two values perfectly align (`ἴσον`).
    Eq,
    /// Tests if the identities of two values are distinct (`ἄνισον`).
    Ne,
    /// Asserts the preceding value is quantitatively lesser (`ἔλαττον`).
    Lt,
    /// Asserts the preceding value is bounded by the subsequent value.
    Le,
    /// Asserts the preceding value is quantitatively greater (`μεῖζον`).
    Gt,
    /// Asserts the preceding value dominates or equates to the subsequent value.
    Ge,
    // Boolean
    /// Conjoins two truths, requiring both to manifest reality (`καί`).
    And,
    /// Offers an alternative path, requiring only one truth to manifest reality (`ἤ`).
    Or,
}

/// Unary operator type for code generation
///
/// Modifies a single expression's existential state or reference.
///
/// ## Examples
///
/// ```rust
/// use glossa::morphology::{UnaryOp, is_negation};
///
/// // The Greek particle "οὐ" negates existence
/// assert!(is_negation("ου"));
/// let not_op = UnaryOp::Not;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Inverts the truth of a statement, stemming from the absolute negation particle (`οὐ`/`οὐκ`).
    Not,
    /// Flips the quantitative sign of an arithmetic value.
    Neg,
    /// Establishes an indirect relationship to an entity, denoting a view rather than ownership.
    Ref,
}
