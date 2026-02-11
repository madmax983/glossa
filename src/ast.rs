//! Abstract Syntax Tree for ΓΛΩΣΣΑ
//!
//! The AST captures the semantic structure of a GLOSSA program,
//! preserving morphological information from Greek words.
//!
//! # The Tree Structure
//!
//! The AST hierarchy reflects the grammatical structure of the language:
//!
//! * [`Program`]: The root node, containing a list of statements.
//! * [`Statement`]: A sentence, ending with a period (`.`) or query mark (`?` / `;`).
//!   * A statement consists of one or more [`Clause`]s.
//! * [`Clause`]: A comma-separated part of a statement.
//!   * Example: `ὁ ἄνθρωπος, τὸν λόγον λέγει.` (Two clauses: "The man", "says the word").
//! * [`Expr`]: An expression (word, literal, operation).
//!   * [`Expr::Word`]: A raw Greek word with its original and normalized forms.
//!
//! # Design Philosophy
//!
//! Unlike traditional ASTs that might discard surface-level details, the GLOSSA AST
//! preserves the *original* Greek text in [`Word`] nodes. This is crucial for:
//!
//! 1. **Error Reporting**: Using the original polytonic Greek in error messages.
//! 2. **Morphological Analysis**: The semantic phase needs the original form to
//!    distinguish subtle variations if needed.
//!
//! # Example
//!
//! A simple program like `«χαῖρε» λέγε.` produces:
//!
//! ```text
//! Program
//! └── Statement::Regular
//!     └── Clause
//!         ├── Expr::StringLiteral("χαῖρε")
//!         └── Expr::Word("λέγε")
//! ```

use smol_str::SmolStr;

/// A complete GLOSSA program
///
/// The root of the Abstract Syntax Tree, containing a sequence of statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// The list of top-level statements in the program.
    pub statements: Vec<Statement>,
}

/// A single statement, ending with . (statement), ? (query), or ; (propagate)
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// A regular statement with clauses
    ///
    /// The most common statement type, representing imperative actions,
    /// bindings, or expressions.
    ///
    /// # Example
    ///
    /// ```glossa
    /// «χαῖρε» λέγε.
    /// ```
    Regular {
        clauses: Vec<Clause>,
        is_query: bool,
        is_propagate: bool,
    },
    /// A type definition statement
    ///
    /// Defines a new struct type.
    ///
    /// # Example
    ///
    /// ```glossa
    /// εἶδος Χρήστης ὁρίζειν {
    ///     ὄνομα ὀνόματος.
    /// }.
    /// ```
    TypeDefinition(TypeDef),
    /// A trait definition statement
    ///
    /// Defines a new interface (trait).
    ///
    /// # Example
    ///
    /// ```glossa
    /// χαρακτήρ Εκτυπώσιμος ὁρίζειν {
    ///     τύπωσις(εαυτός).
    /// }.
    /// ```
    TraitDefinition(TraitDef),
    /// A trait implementation statement
    ///
    /// Implements a trait for a specific type.
    ///
    /// # Example
    ///
    /// ```glossa
    /// εἶδος Χρήστης τῷ Εκτυπώσιμος ἐμπίπτειν {
    ///     τύπωσις(εαυτός) {
    ///         εαυτοῦ ὄνομα λέγε.
    ///     }
    /// }.
    /// ```
    TraitImpl(TraitImplDef),
    /// A test declaration
    ///
    /// Defines a unit test.
    ///
    /// # Example
    ///
    /// ```glossa
    /// δοκιμή «test name».
    ///     ξ 5 ἰσοῦται.
    /// τέλος.
    /// ```
    TestDeclaration(TestDecl),
}

/// A type definition (struct)
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: Word,
    pub fields: Vec<FieldDecl>,
}

/// A field declaration in a type
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub name: Word,
    pub type_name: Word,
}

/// A trait definition
#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    pub name: Word,
    pub methods: Vec<TraitMethodDecl>,
}

/// A method declaration in a trait
#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethodDecl {
    pub name: Word,
    pub params: Vec<FieldDecl>,
    pub is_default: bool,
    pub body: Option<Vec<Statement>>,
}

/// A trait implementation
#[derive(Debug, Clone, PartialEq)]
pub struct TraitImplDef {
    pub type_name: Word,
    pub trait_name: Word,
    pub methods: Vec<ImplMethodDef>,
}

/// A method implementation in a trait impl
#[derive(Debug, Clone, PartialEq)]
pub struct ImplMethodDef {
    pub name: Word,
    pub params: Vec<FieldDecl>,
    pub body: Vec<Statement>,
}

/// A test declaration (δοκιμή)
#[derive(Debug, Clone, PartialEq)]
pub struct TestDecl {
    /// Test name (string literal)
    pub name: String,
    /// Test body statements
    pub body: Vec<Statement>,
}

/// A clause within a statement (comma-separated)
#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    /// Expressions in this clause (chained with middle dot)
    pub expressions: Vec<Expr>,
}

impl Statement {
    /// Get all expressions flattened (for backwards compatibility)
    pub fn expressions(&self) -> Box<dyn Iterator<Item = &Expr> + '_> {
        match self {
            Statement::Regular { clauses, .. } => {
                Box::new(clauses.iter().flat_map(|c| c.expressions.iter()))
            }
            Statement::TypeDefinition(_) => Box::new(std::iter::empty()),
            Statement::TraitDefinition(_) => Box::new(std::iter::empty()),
            Statement::TraitImpl(_) => Box::new(std::iter::empty()),
            Statement::TestDeclaration(_) => Box::new(std::iter::empty()),
        }
    }

    /// Check if this is a query statement
    pub fn is_query(&self) -> bool {
        match self {
            Statement::Regular { is_query, .. } => *is_query,
            Statement::TypeDefinition(_) => false,
            Statement::TraitDefinition(_) => false,
            Statement::TraitImpl(_) => false,
            Statement::TestDeclaration(_) => false,
        }
    }

    /// Check if this is a propagate statement (ends with `;`)
    pub fn is_propagate(&self) -> bool {
        match self {
            Statement::Regular { is_propagate, .. } => *is_propagate,
            Statement::TypeDefinition(_) => false,
            Statement::TraitDefinition(_) => false,
            Statement::TraitImpl(_) => false,
            Statement::TestDeclaration(_) => false,
        }
    }

    /// Get clauses if this is a regular statement
    pub fn clauses(&self) -> &[Clause] {
        match self {
            Statement::Regular { clauses, .. } => clauses,
            Statement::TypeDefinition(_) => &[],
            Statement::TraitDefinition(_) => &[],
            Statement::TraitImpl(_) => &[],
            Statement::TestDeclaration(_) => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_new_normalization() {
        let w = Word::new("Άνθρωπος");
        assert_eq!(w.original, "Άνθρωπος");
        assert_eq!(w.normalized, "ανθρωπος");
    }

    #[test]
    fn test_word_new_simple() {
        let w = Word::new("test");
        assert_eq!(w.original, "test");
        assert_eq!(w.normalized, "test");
    }
}

/// An expression in GLOSSA
///
/// Expressions represent values that can be evaluated.
/// They include literals, variable references, operations, and function calls.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A string literal: «text»
    ///
    /// # Example
    /// `«χαῖρε κόσμε»`
    StringLiteral(String),

    /// A number literal
    ///
    /// # Example
    /// `42` or `πέντε`
    NumberLiteral(i64),

    /// A boolean literal: ἀληθές or ψεῦδος
    BooleanLiteral(bool),

    /// An array literal: `[1, 2, 3]`
    ArrayLiteral(Vec<Expr>),

    /// Index access: `array[index]`
    IndexAccess { array: Box<Expr>, index: Box<Expr> },

    /// A single Greek word with morphological information
    ///
    /// This is the most basic building block. The semantic analyzer uses
    /// the morphological information to determine if this word is a
    /// variable, a keyword, or part of a larger phrase.
    Word(Word),

    /// Multiple terms forming a phrase
    ///
    /// Phrases are sequences of words that haven't been fully parsed into
    /// specific grammatical structures yet.
    Phrase(Vec<Expr>),

    /// A property access (genitive construction)
    ///
    /// # Example
    /// `χρήστου ὄνομα` (the user's name)
    PropertyAccess {
        owner: Box<Expr>,
        property: Box<Expr>,
    },

    /// A function/verb call
    ///
    /// # Example
    /// `λέγε(«χαῖρε»)` (Explicit call syntax)
    Call { verb: Word, arguments: Vec<Expr> },

    /// Variable binding (ἔστω construction)
    ///
    /// # Example
    /// `ξ πέντε ἔστω` -> `Binding { name: "ξ", value: 5 }`
    Binding { name: Word, value: Box<Expr> },

    /// Binary operation (arithmetic, comparison, boolean)
    ///
    /// # Example
    /// `x μεῖζον y` -> `x > y`
    BinOp {
        left: Box<Expr>,
        op: BinOperator,
        right: Box<Expr>,
    },

    /// Unary operation (negation)
    ///
    /// # Example
    /// `οὐκ x` -> `!x`
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expr>,
    },

    /// A block of statements in braces { ... }
    Block(Vec<Statement>),
}

/// Binary operators in GLOSSA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOperator {
    // Arithmetic
    Add, // ἄθροισμα
    Sub, // διαφορά
    Mul, // γινόμενον
    Div, // μέρος
    Mod, // ὑπόλοιπον

    // Comparison
    Eq, // ἴσον
    Ne, // ἄνισον
    Lt, // ἔλαττον
    Le, // ἔλαττον ἢ ἴσον
    Gt, // μεῖζον
    Ge, // μεῖζον ἢ ἴσον

    // Boolean
    And, // καί
    Or,  // ἤ
}

/// Unary operators in GLOSSA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,    // οὐ/οὐκ/οὐχ - logical negation
    Neg,    // arithmetic negation
    Unwrap, // ! - confident extraction from Option/Result
}

/// A Greek word with original and normalized forms
///
/// This struct preserves the original polytonic Greek text (for display)
/// while providing a normalized version for compiler analysis.
///
/// # Examples
///
/// ```
/// use glossa::ast::Word;
///
/// let word = Word::new("Ἀθῆναι");
/// assert_eq!(word.original, "Ἀθῆναι");
/// assert_eq!(word.normalized, "αθηναι");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    /// Original text with diacritics
    pub original: SmolStr,
    /// Normalized (lowercase, no diacritics)
    pub normalized: SmolStr,
}

impl Word {
    /// Create a new word, automatically generating the normalized form
    pub fn new(original: impl Into<SmolStr>) -> Self {
        let original = original.into();
        let normalized = crate::text::normalize_greek(&original);
        Word {
            original,
            normalized,
        }
    }
}
