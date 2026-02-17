//! Data models for the semantic assembler
//!
//! This module contains the data structures used by the `Assembler` to represent
//! the state of a sentence as it is being assembled. It separates the data
//! representation from the assembly logic.

use crate::ast::Expr;
use crate::morphology::lexicon::BinaryOp;
use crate::morphology::{Case, Gender, Mood, Number, Person, Tense, Voice};
use smol_str::SmolStr;

/// A fully assembled statement with all grammatical roles filled
///
/// This struct represents the "final state" of a sentence after parsing.
/// It contains all the semantic components (subject, verb, object, etc.)
/// extracted from the input stream.
#[derive(Debug, Clone, Default)]
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

/// A noun/pronoun constituent with its grammatical info
#[derive(Debug, Clone)]
pub struct Constituent {
    /// The dictionary form
    pub lemma: SmolStr,

    /// Original text as it appeared
    pub original: SmolStr,

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
#[derive(Debug, Clone)]
pub struct VerbConstituent {
    /// The dictionary form
    pub lemma: SmolStr,

    /// Original text as it appeared
    pub original: SmolStr,

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
#[derive(Debug, Clone)]
pub struct ParticipleConstituent {
    /// The verb stem extracted from the participle
    pub verb_lemma: SmolStr,
    /// Original text as it appeared
    pub original: SmolStr,
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
