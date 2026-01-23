//! AST node definitions for ΓΛΩΣΣΑ
//!
//! These nodes capture the structure of a GLOSSA program,
//! preserving both original Greek text and normalized forms.


/// A complete GLOSSA program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// A single statement, ending with . (statement) or ? (query)
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    /// The expressions in this statement (may be chained with ·)
    pub expressions: Vec<Expr>,
    /// Whether this is a query (ends with ?)
    pub is_query: bool,
}

/// An expression in GLOSSA
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A string literal: «text»
    StringLiteral(String),

    /// A number literal
    NumberLiteral(i64),

    /// A boolean literal: ἀληθές or ψεῦδος
    BooleanLiteral(bool),

    /// A single Greek word with morphological information
    Word(Word),

    /// Multiple terms forming a phrase
    Phrase(Vec<Expr>),

    /// A property access (genitive construction)
    /// e.g., χρήστου ὄνομα = "the name of the user"
    PropertyAccess {
        owner: Box<Expr>,
        property: Box<Expr>,
    },

    /// A function/verb call
    Call {
        verb: Word,
        arguments: Vec<Expr>,
    },

    /// Variable binding (ἔστω construction)
    Binding {
        name: Word,
        value: Box<Expr>,
    },
}

/// A Greek word with original and normalized forms
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    /// Original text with diacritics
    pub original: String,
    /// Normalized (lowercase, no diacritics)
    pub normalized: String,
}

impl Word {
    pub fn new(original: impl Into<String>) -> Self {
        let original = original.into();
        let normalized = crate::grammar::normalize_greek(&original);
        Word { original, normalized }
    }
}

/// Analyzed word with morphological information
#[derive(Debug, Clone, PartialEq)]
pub struct AnalyzedWord {
    pub word: Word,
    pub case: Option<crate::morphology::Case>,
    pub number: Option<crate::morphology::Number>,
    pub gender: Option<crate::morphology::Gender>,
    pub person: Option<crate::morphology::Person>,
    pub tense: Option<crate::morphology::Tense>,
    pub mood: Option<crate::morphology::Mood>,
    pub voice: Option<crate::morphology::Voice>,
}

impl From<Word> for AnalyzedWord {
    fn from(word: Word) -> Self {
        AnalyzedWord {
            word,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
        }
    }
}
