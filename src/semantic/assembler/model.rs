//! Data models for the semantic assembler
//!
//! This module contains the data structures representing the assembled
//! sentence components.

use crate::ast::Expr;
use crate::morphology::lexicon::BinaryOp;
use crate::morphology::{Case, Gender, Mood, Number, Person, Tense, Voice};

/// A fully assembled statement with all grammatical roles filled
///
/// This struct represents the "final state" of a sentence after parsing.
/// It contains all the semantic components (subject, verb, object, etc.)
/// extracted from the input stream.
#[derive(Debug, Clone)]
pub struct AssembledStatement {
    /// The subject (nominative) - the agent/doer
    ///
    /// Example: **ὁ ἄνθρωπος** λέγει (The **man** speaks)
    pub subject: Option<Constituent>,

    /// Additional nominatives (for function names, etc.)
    ///
    /// Used in patterns where multiple nominatives appear, such as
    /// function calls or predicate nominatives.
    pub nominatives: Vec<Constituent>,

    /// The verb - the action
    ///
    /// Example: ὁ ἄνθρωπος **λέγει** (The man **speaks**)
    pub verb: Option<VerbConstituent>,

    /// The direct object (accusative) - receives the action
    ///
    /// Example: βλέπω **τὸν ἄνθρωπον** (I see **the man**)
    pub object: Option<Constituent>,
    /// The indirect object (dative) - recipient/beneficiary
    ///
    /// Example: δίδωμι **τῷ ἀνθρώπῳ** (I give **to the man**)
    pub indirect: Option<Constituent>,

    /// Possessors/sources (genitive) - attached to other constituents
    ///
    /// Example: τὸ τοῦ **ἀνθρώπου** βιβλίον (The **man's** book)
    pub genitives: Vec<Constituent>,

    /// Adjectives modifying nouns (νέον for "new")
    ///
    /// These are accumulated and applied to the relevant nouns during semantic analysis.
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
    /// Whether this binding has the mutable marker (μετά)
    pub has_mutable_marker: bool,
    /// Whether this statement has the containment preposition (ἐν)
    /// Used for contains patterns: element ἐν set? → set.contains(&element)
    pub has_containment_preposition: bool,
    /// Whether this statement has the delimiter preposition (κατά)
    /// Used for split/join patterns: string κατά delimiter σχίζεται → string.split(delimiter)
    pub has_delimiter_preposition: bool,
    /// String method call: (method_name, delimiter)
    /// Used for split/join patterns
    pub string_method: Option<(String, String)>,
}

/// A noun/pronoun constituent with its grammatical info
///
/// This represents a word that fills a nominal slot (Subject, Object, etc.).
/// It carries both its original surface form and its normalized lemma.
#[derive(Debug, Clone)]
pub struct Constituent {
    /// The dictionary form
    ///
    /// Used for semantic analysis and code generation.
    /// Example: "ανθρωπος" (from "ἄνθρωπος")
    pub lemma: String,

    /// Original text as it appeared
    ///
    /// Preserved for error messages and display purposes.
    /// Example: "ἄνθρωπος"
    pub original: String,

    /// Grammatical case
    ///
    /// Determines which slot this constituent fills:
    /// - `Nominative` -> Subject
    /// - `Accusative` -> Object
    /// - `Dative` -> Indirect Object
    pub case: Case,

    /// Grammatical number
    ///
    /// Used for subject-verb agreement checks.
    pub number: Option<Number>,

    /// Grammatical gender
    ///
    /// Used for adjective-noun agreement checks.
    pub gender: Option<Gender>,
}

/// A verb constituent with its grammatical info
///
/// This represents the main action of the sentence.
#[derive(Debug, Clone)]
pub struct VerbConstituent {
    /// The dictionary form (1st person singular present)
    ///
    /// Example: "λεγω" (from "λέγει")
    pub lemma: String,

    /// Original text as it appeared
    ///
    /// Example: "λέγει"
    pub original: String,

    /// Person (1st, 2nd, 3rd)
    ///
    /// Used for agreement with the subject.
    pub person: Option<Person>,

    /// Number (singular, plural)
    ///
    /// Used for agreement with the subject.
    pub number: Option<Number>,

    /// Tense (present, aorist, etc.)
    ///
    /// Determines execution semantics (e.g., Aorist = immediate, Present = continuous).
    pub tense: Option<Tense>,

    /// Mood (indicative, imperative, etc.)
    ///
    /// Determines sentence type (Statement vs Command).
    pub mood: Option<Mood>,

    /// Voice (active, middle, passive)
    ///
    /// Determines the relationship between subject and action.
    /// - Active: Subject does action
    /// - Middle: Subject acts on self/for self (often used for specific ops like `.pop()`)
    pub voice: Option<Voice>,
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
