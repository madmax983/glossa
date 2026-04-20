use crate::ast::Expr;
use crate::morphology::{Gender, Number, Person, Case, Voice, Mood, Tense};
use crate::morphology::lexicon::BinaryOp;
use smol_str::SmolStr;

/// A fully assembled statement with all grammatical roles filled
///
/// This struct represents the intermediate "assembled" state of a sentence after
/// grammatical parsing but before semantic classification. It acts as a set of labeled
/// buckets (Subject, Verb, Object, Literals, etc.) where words are sorted based purely on
/// their Ancient Greek morphology.
///
/// # Why it exists
/// Ancient Greek relies on word endings (morphology) rather than word order to dictate meaning.
/// Therefore, the `Assembler` iterates through words in any order, analyzes their morphology,
/// and places them into the correct bucket in this struct.
/// Later, the `Semantic Analyzer` looks at which buckets are filled to determine if this is
/// an assignment, a function call, a loop, or a print statement.
///
/// ## Examples
///
/// If we parse the sentence "the user says the name" (`ὁ χρήστης τὸ ὄνομα λέγει`):
///
/// ```rust
/// use glossa::semantic::{AssembledStatement, Constituent};
/// use smol_str::SmolStr;
/// use glossa::morphology::{Case, Number, Gender, Person};
///
/// let mut stmt = AssembledStatement::default();
///
/// // Simulating "the user" (ὁ χρήστης)
/// stmt.subject = Some(Constituent {
///     lemma: SmolStr::new("χρηστης"),
///     original: SmolStr::new("χρήστης"),
///     normalized: SmolStr::new("χρηστης"),
///     case: Case::Nominative,
///     number: Some(Number::Singular),
///     gender: Some(Gender::Masculine),
///     person: Some(Person::Third),
/// });
///
/// assert!(stmt.subject.is_some());
/// ```
#[derive(Clone, Default)]
pub struct AssembledStatement {
    /// The subject (nominative) - the agent/doer
    pub subject: Option<Constituent>,
    /// Additional nominatives (for function names, etc.)
    pub nominatives: Vec<Constituent>,
    /// The verb - the action
    pub verb: Option<VerbConstituent>,
    /// The direct object (accusative) - receives the action
    pub object: Option<Constituent>,
    /// The indirect object (dative) - recipient/beneficiary
    pub indirect: Option<Constituent>,
    /// Possessors/sources (genitive) - attached to other constituents
    pub genitives: Vec<Constituent>,
    /// Adjectives modifying nouns
    pub adjectives: Vec<Constituent>,
    /// Literal values (strings, numbers) that appeared
    pub literals: Vec<Literal>,
    /// Array literals that appeared
    pub arrays: Vec<Vec<Expr>>,
    /// Index accesses (array, index)
    pub index_accesses: Vec<(Expr, Expr)>,
    /// Property accesses (owner, property)
    pub property_accesses: Vec<(String, String)>,
    /// Binary operators found between expressions
    pub operators: Vec<BinaryOp>,
    /// Parenthesized blocks (nested expressions)
    pub blocks: Vec<Vec<crate::ast::Statement>>,
    /// Nested phrases (parenthesized function calls)
    pub nested_phrases: Vec<Vec<Expr>>,
    /// Participles (used for lambdas/closures)
    pub participles: Vec<ParticipleConstituent>,
    /// Unwrap expressions (expr!)
    pub unwraps: Vec<Expr>,
    /// Whether this is a query (ends with ?)
    pub is_query: bool,
    /// Whether this statement propagates (ends with ;)
    pub is_propagate: bool,
    /// Whether this binding has the mutable marker (μετά)
    pub has_mutable_marker: bool,
    /// Whether this statement has the containment preposition (ἐν)
    pub has_containment_preposition: bool,
    /// Whether this statement has the delimiter preposition (κατά)
    pub has_delimiter_preposition: bool,
    /// String method call: (method_name, delimiter)
    pub string_method: Option<(String, String)>,
}

/// A literal value
///
/// Literals are atomic truths inscribed directly by the user. They require no
/// variables or references to evaluate, possessing their complete essence inherently.
///
/// ## Examples
///
/// ```rust
/// use glossa::semantic::Literal;
///
/// // Create a string literal embodying truth
/// let text = Literal::String("Ἀλήθεια".to_string());
///
/// // A perfect number literal
/// let perfect_number = Literal::Number(6);
/// ```
#[derive(Clone)]
pub enum Literal {
    /// A quoted string literal containing raw text
    String(String),
    /// An integer numeric literal or parsed greek numeral
    Number(i64),
    /// A true/false boolean literal (`ἀληθές` or `ψεῦδος`)
    Boolean(bool),
}

/// A noun/pronoun constituent with its grammatical info
///
/// In the grammatical world of ΓΛΩΣΣΑ, a `Constituent` is the physical manifestation of a noun,
/// pronoun, or adjective acting as a primary sentence component (e.g., Subject, Object).
/// It bridges the raw morphological analysis ([`crate::morphology::models::MorphAnalysis`])
/// with the structural needs of the [`Assembler`].
///
/// # Why it exists
/// The parser gives us isolated words, but to build a sentence, we need to know *what* each word is doing.
/// The `Constituent` holds onto both the raw text (for error reporting) and the deep grammatical truths
/// (like [`Case`] and [`Number`]) required to verify things like Subject-Verb Agreement.
///
/// ## Examples
///
/// Creating a constituent representing the subject "the man" (`ὁ ἄνθρωπος`):
///
/// ```rust
/// use glossa::semantic::Constituent;
/// use smol_str::SmolStr;
/// use glossa::morphology::{Case, Number, Gender, Person};
///
/// let subject = Constituent {
///     lemma: SmolStr::new("ανθρωπος"),
///     original: SmolStr::new("ἄνθρωπος"),
///     normalized: SmolStr::new("ανθρωπος"),
///     case: Case::Nominative,
///     number: Some(Number::Singular),
///     gender: Some(Gender::Masculine),
///     person: Some(Person::Third),
/// };
///
/// assert_eq!(subject.case, Case::Nominative);
/// ```
#[derive(Clone)]
#[allow(dead_code)]
pub struct Constituent {
    /// The dictionary form (e.g., "ανθρωπος" for "ἀνθρώπους").
    /// This is what the semantic resolver looks up in the [`crate::semantic::resolver::Scope`].
    pub lemma: SmolStr,
    /// Original text as it appeared
    pub original: SmolStr,
    /// Normalized text (lowercase, no diacritics)
    pub normalized: SmolStr,
    /// Grammatical case
    pub case: Case,
    /// Grammatical number
    pub number: Option<Number>,
    /// Grammatical gender
    pub gender: Option<Gender>,
    /// Grammatical person (1st, 2nd, 3rd)
    pub person: Option<Person>,
}

/// A verb constituent with its grammatical info
#[derive(Clone)]
#[allow(dead_code)]
pub struct VerbConstituent {
    /// The dictionary form
    pub lemma: SmolStr,
    /// Original text as it appeared
    pub original: SmolStr,
    /// Normalized text (lowercase, no diacritics)
    pub normalized: SmolStr,
    /// Person (1st, 2nd, 3rd)
    pub person: Option<Person>,
    /// Number (singular, plural)
    pub number: Option<Number>,
    /// Tense (present, aorist, etc.)
    pub tense: Option<Tense>,
    /// Mood (indicative, imperative, etc.)
    pub mood: Option<Mood>,
    /// Voice (active, middle, passive)
    pub voice: Option<Voice>,
}

/// A participle constituent (used for lambdas/closures)
#[derive(Clone)]
#[allow(dead_code)]
pub struct ParticipleConstituent {
    /// The verb stem extracted from the participle
    pub verb_lemma: SmolStr,
    /// Original text as it appeared
    pub original: SmolStr,
    /// Normalized text (lowercase, no diacritics)
    pub normalized: SmolStr,
    /// Tense (present, aorist, perfect)
    pub tense: Tense,
    /// Voice (active, middle, passive)
    pub voice: Voice,
    /// Case (adjectival property)
    pub case: Case,
    /// Gender (adjectival property)
    pub gender: Gender,
    /// Number (adjectival property)
    pub number: Number,
}

impl std::fmt::Debug for AssembledStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            f.debug_struct("AssembledStatement")
                .field("subject", &self.subject)
                .field("nominatives", &self.nominatives)
                .field("verb", &self.verb)
                .field("object", &self.object)
                .field("indirect", &self.indirect)
                .field("genitives", &self.genitives)
                .field("adjectives", &self.adjectives)
                .field("literals", &self.literals)
                .field("arrays", &self.arrays)
                .field("index_accesses", &self.index_accesses)
                .field("property_accesses", &self.property_accesses)
                .field("operators", &self.operators)
                .field("blocks", &self.blocks)
                .field("nested_phrases", &self.nested_phrases)
                .field("participles", &self.participles)
                .field("unwraps", &self.unwraps)
                .field("is_query", &self.is_query)
                .field("is_propagate", &self.is_propagate)
                .field("has_mutable_marker", &self.has_mutable_marker)
                .field(
                    "has_containment_preposition",
                    &self.has_containment_preposition,
                )
                .field("has_delimiter_preposition", &self.has_delimiter_preposition)
                .field("string_method", &self.string_method)
                .finish()
        })
    }
}

impl std::fmt::Debug for Constituent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            f.debug_struct("Constituent")
                .field("lemma", &self.lemma)
                .field("original", &self.original)
                .field("normalized", &self.normalized)
                .field("case", &self.case)
                .field("number", &self.number)
                .field("gender", &self.gender)
                .field("person", &self.person)
                .finish()
        })
    }
}

impl std::fmt::Debug for VerbConstituent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            f.debug_struct("VerbConstituent")
                .field("lemma", &self.lemma)
                .field("original", &self.original)
                .field("normalized", &self.normalized)
                .field("person", &self.person)
                .field("number", &self.number)
                .field("tense", &self.tense)
                .field("mood", &self.mood)
                .field("voice", &self.voice)
                .finish()
        })
    }
}

impl std::fmt::Debug for ParticipleConstituent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            f.debug_struct("ParticipleConstituent")
                .field("verb_lemma", &self.verb_lemma)
                .field("original", &self.original)
                .field("normalized", &self.normalized)
                .field("tense", &self.tense)
                .field("voice", &self.voice)
                .field("case", &self.case)
                .field("gender", &self.gender)
                .field("number", &self.number)
                .finish()
        })
    }
}

impl std::fmt::Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            Literal::String(s) => f.debug_tuple("String").field(s).finish(),
            Literal::Number(n) => f.debug_tuple("Number").field(n).finish(),
            Literal::Boolean(b) => f.debug_tuple("Boolean").field(b).finish(),
        })
    }
}
