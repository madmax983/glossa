//! AST node definitions for ΓΛΩΣΣΑ
//!
//! These nodes capture the structure of a GLOSSA program,
//! preserving both original Greek text and normalized forms.

use smol_str::SmolStr;

/// A complete GLOSSA program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// A single statement, ending with . (statement), ? (query), or ; (propagate)
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// A regular statement with clauses
    Regular {
        clauses: Vec<Clause>,
        is_query: bool,
        is_propagate: bool,
    },
    /// A type definition statement
    TypeDefinition(TypeDef),
    /// A trait definition statement
    TraitDefinition(TraitDef),
    /// A trait implementation statement
    TraitImpl(TraitImplDef),
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
        }
    }

    /// Check if this is a query statement
    pub fn is_query(&self) -> bool {
        match self {
            Statement::Regular { is_query, .. } => *is_query,
            Statement::TypeDefinition(_) => false,
            Statement::TraitDefinition(_) => false,
            Statement::TraitImpl(_) => false,
        }
    }

    /// Check if this is a propagate statement (ends with `;`)
    pub fn is_propagate(&self) -> bool {
        match self {
            Statement::Regular { is_propagate, .. } => *is_propagate,
            Statement::TypeDefinition(_) => false,
            Statement::TraitDefinition(_) => false,
            Statement::TraitImpl(_) => false,
        }
    }

    /// Get clauses if this is a regular statement
    pub fn clauses(&self) -> &[Clause] {
        match self {
            Statement::Regular { clauses, .. } => clauses,
            Statement::TypeDefinition(_) => &[],
            Statement::TraitDefinition(_) => &[],
            Statement::TraitImpl(_) => &[],
        }
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
    Word(Word),

    /// Multiple terms forming a phrase
    ///
    /// Phrases are sequences of words that haven't been fully parsed into
    /// specific grammatical structures yet.
    Phrase(Vec<Expr>),

    /// A property access (genitive construction)
    /// e.g., χρήστου ὄνομα = "the name of the user"
    PropertyAccess {
        owner: Box<Expr>,
        property: Box<Expr>,
    },

    /// A function/verb call
    Call { verb: Word, arguments: Vec<Expr> },

    /// Variable binding (ἔστω construction)
    Binding { name: Word, value: Box<Expr> },

    /// Binary operation (arithmetic, comparison, boolean)
    /// e.g., x μεῖζον y → x > y
    BinOp {
        left: Box<Expr>,
        op: BinOperator,
        right: Box<Expr>,
    },

    /// Unary operation (negation)
    /// e.g., οὐκ x → !x
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expr>,
    },

    /// A block of statements in braces { ... }
    Block(Vec<Statement>),
}

/// Capture mode for closures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    /// Borrow captured variables (default for present participles)
    Borrow,

    /// Move captured variables (for aorist participles)
    Move,

    /// Memoize result (for perfect participles)
    Memoize,
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
        let normalized = crate::grammar::normalize_greek(&original);
        Word {
            original,
            normalized,
        }
    }
}
