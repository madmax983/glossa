//! Constituent types for sentence assembly
//!
//! These types represent the grammatical components of a statement
//! as they are being assembled.

use crate::ast::Expr;
use crate::morphology::lexicon::BinaryOp;
use crate::morphology::{Case, Gender, Mood, Number, Person, Tense, Voice};

/// A fully assembled statement with all grammatical roles filled
#[derive(Debug, Clone)]
pub struct AssembledStatement {
    /// The subject (nominative) - the agent/doer
    /// Can have multiple nominatives for function call patterns
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
    /// Adjectives modifying nouns (νέον for "new")
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
    /// Whether this statement propagates (ends with ;) - converts to `?` in Rust
    pub is_propagate: bool,
}

/// A noun/pronoun constituent with its grammatical info
#[derive(Debug, Clone)]
pub struct Constituent {
    /// The dictionary form
    pub lemma: String,
    /// Original text as it appeared
    pub original: String,
    /// Grammatical case
    pub case: Case,
    /// Grammatical number
    pub number: Option<Number>,
    /// Grammatical gender
    pub gender: Option<Gender>,
}

/// A verb constituent with its grammatical info
#[derive(Debug, Clone)]
pub struct VerbConstituent {
    /// The dictionary form (1st person singular present)
    pub lemma: String,
    /// Original text as it appeared
    pub original: String,
    /// Person (1st, 2nd, 3rd)
    pub person: Option<Person>,
    /// Number (singular, plural)
    pub number: Option<Number>,
    /// Tense (present, aorist, etc.)
    pub tense: Option<Tense>,
    /// Mood (indicative, imperative, etc.)
    pub mood: Option<Mood>,
}

/// A participle constituent (used for lambdas/closures)
#[derive(Debug, Clone)]
pub struct ParticipleConstituent {
    /// The verb stem extracted from the participle
    pub verb_lemma: String,
    /// Original text as it appeared
    pub original: String,
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

/// A literal value
#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(i64),
    Boolean(bool),
}

/// Errors that can occur during assembly
#[derive(Debug, Clone, thiserror::Error)]
pub enum AssemblyError {
    #[error("Διπλοῦν ὑποκείμενον! Δύο βασιλεῖς οὐ δύνανται μιᾶς πόλεως ἄρχειν.")]
    DoubleSubject,

    #[error("Διπλοῦν ἀντικείμενον! Ἓν μόνον κατηγορεῖς.")]
    DoubleObject,

    #[error("Διπλοῦν ῥῆμα! Μία πρᾶξις ἑκάστοτε.")]
    DoubleVerb,

    #[error("Ῥῆμα οὐχ εὑρέθη! Οὐδὲν ἐγένετο.")]
    MissingVerb,

    #[error("Ἀσυμφωνία: ὑποκείμενον {subject:?} ἀλλὰ ῥῆμα {verb:?}")]
    SubjectVerbDisagreement {
        subject: (Option<Person>, Option<Number>),
        verb: (Option<Person>, Option<Number>),
    },

    #[error("Ἀσυμφωνία γένους: {word1} ({gender1:?}) πρὸς {word2} ({gender2:?})")]
    GenderMismatch {
        word1: String,
        gender1: Gender,
        word2: String,
        gender2: Gender,
    },
}
