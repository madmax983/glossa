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
    /// A test declaration
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
    fn test_word_new() {
        let w = Word::new("Λόγος");
        assert_eq!(w.original, "Λόγος");
        assert_eq!(w.normalized, "λογος");
    }

    #[test]
    fn test_statement_helpers() {
        let stmt = Statement::Regular {
            clauses: vec![],
            is_query: true,
            is_propagate: false,
        };
        assert!(stmt.is_query());
        assert!(!stmt.is_propagate());
        assert!(stmt.clauses().is_empty());
        assert_eq!(stmt.expressions().count(), 0);

        let stmt_prop = Statement::Regular {
            clauses: vec![],
            is_query: false,
            is_propagate: true,
        };
        assert!(!stmt_prop.is_query());
        assert!(stmt_prop.is_propagate());

        let type_def = Statement::TypeDefinition(TypeDef {
            name: Word::new("T"),
            fields: vec![],
        });
        assert!(!type_def.is_query());
        assert!(!type_def.is_propagate());
        assert!(type_def.clauses().is_empty());
        assert_eq!(type_def.expressions().count(), 0);

        let trait_def = Statement::TraitDefinition(TraitDef {
            name: Word::new("Tr"),
            methods: vec![],
        });
        assert!(!trait_def.is_query());
        assert!(!trait_def.is_propagate());
        assert!(trait_def.clauses().is_empty());
        assert_eq!(trait_def.expressions().count(), 0);

        let trait_impl = Statement::TraitImpl(TraitImplDef {
            type_name: Word::new("T"),
            trait_name: Word::new("Tr"),
            methods: vec![],
        });
        assert!(!trait_impl.is_query());
        assert!(!trait_impl.is_propagate());
        assert!(trait_impl.clauses().is_empty());
        assert_eq!(trait_impl.expressions().count(), 0);

        let test_decl = Statement::TestDeclaration(TestDecl {
            name: "test".into(),
            body: vec![],
        });
        assert!(!test_decl.is_query());
        assert!(!test_decl.is_propagate());
        assert!(test_decl.clauses().is_empty());
        assert_eq!(test_decl.expressions().count(), 0);
    }

    #[test]
    fn test_statement_expressions_iterator() {
        let stmt = Statement::Regular {
            clauses: vec![Clause {
                expressions: vec![Expr::NumberLiteral(1), Expr::NumberLiteral(2)],
            }],
            is_query: false,
            is_propagate: false,
        };
        let exprs: Vec<_> = stmt.expressions().collect();
        assert_eq!(exprs.len(), 2);
        assert_eq!(exprs[0], &Expr::NumberLiteral(1));
        assert_eq!(exprs[1], &Expr::NumberLiteral(2));
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
