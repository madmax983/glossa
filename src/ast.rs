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
#[derive(Clone, PartialEq)]
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

impl std::fmt::Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            Statement::Regular {
                clauses,
                is_query,
                is_propagate,
            } => f
                .debug_struct("Regular")
                .field("clauses", clauses)
                .field("is_query", is_query)
                .field("is_propagate", is_propagate)
                .finish(),
            Statement::TypeDefinition(def) => f.debug_tuple("TypeDefinition").field(def).finish(),
            Statement::TraitDefinition(def) => f.debug_tuple("TraitDefinition").field(def).finish(),
            Statement::TraitImpl(def) => f.debug_tuple("TraitImpl").field(def).finish(),
            Statement::TestDeclaration(def) => f.debug_tuple("TestDeclaration").field(def).finish(),
        })
    }
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

/// An expression in GLOSSA
///
/// Expressions represent values that can be evaluated.
/// They include literals, variable references, operations, and function calls.
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
    ///
    /// # Example
    /// `ἀληθές` (True), `ψεῦδος` (False)
    BooleanLiteral(bool),

    /// An array literal: `[1, 2, 3]`
    ///
    /// # Example
    /// `[1, 2, 3]`
    ArrayLiteral(Vec<Expr>),

    /// Index access: `array[index]`
    ///
    /// # Example
    /// `πίναξ[0]`
    IndexAccess { array: Box<Expr>, index: Box<Expr> },

    /// A single Greek word with morphological information
    ///
    /// This is the most basic building block. The semantic analyzer uses
    /// the morphological information to determine if this word is a
    /// variable, a keyword, or part of a larger phrase.
    ///
    /// Unlike [`Phrase`](Expr::Phrase), a `Word` has been parsed as a single unit.
    Word(Word),

    /// Multiple terms forming a phrase
    ///
    /// Phrases are sequences of words that haven't been fully parsed into
    /// specific grammatical structures yet (e.g. parenthesized expressions).
    ///
    /// # Example
    /// `(ὁ ἄνθρωπος)`
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

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            Expr::StringLiteral(s) => f.debug_tuple("StringLiteral").field(s).finish(),
            Expr::NumberLiteral(n) => f.debug_tuple("NumberLiteral").field(n).finish(),
            Expr::BooleanLiteral(b) => f.debug_tuple("BooleanLiteral").field(b).finish(),
            Expr::ArrayLiteral(v) => f.debug_tuple("ArrayLiteral").field(v).finish(),
            Expr::IndexAccess { array, index } => f
                .debug_struct("IndexAccess")
                .field("array", array)
                .field("index", index)
                .finish(),
            Expr::Word(w) => f.debug_tuple("Word").field(w).finish(),
            Expr::Phrase(v) => f.debug_tuple("Phrase").field(v).finish(),
            Expr::PropertyAccess { owner, property } => f
                .debug_struct("PropertyAccess")
                .field("owner", owner)
                .field("property", property)
                .finish(),
            Expr::Call { verb, arguments } => f
                .debug_struct("Call")
                .field("verb", verb)
                .field("arguments", arguments)
                .finish(),
            Expr::Binding { name, value } => f
                .debug_struct("Binding")
                .field("name", name)
                .field("value", value)
                .finish(),
            Expr::BinOp { left, op, right } => f
                .debug_struct("BinOp")
                .field("left", left)
                .field("op", op)
                .field("right", right)
                .finish(),
            Expr::UnaryOp { op, operand } => f
                .debug_struct("UnaryOp")
                .field("op", op)
                .field("operand", operand)
                .finish(),
            Expr::Block(stmts) => f.debug_tuple("Block").field(stmts).finish(),
        })
    }
}

impl Clone for Expr {
    fn clone(&self) -> Self {
        // Prevent stack overflow on deep cloning
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            Expr::StringLiteral(s) => Expr::StringLiteral(s.clone()),
            Expr::NumberLiteral(n) => Expr::NumberLiteral(*n),
            Expr::BooleanLiteral(b) => Expr::BooleanLiteral(*b),
            Expr::ArrayLiteral(v) => Expr::ArrayLiteral(v.clone()),
            Expr::IndexAccess { array, index } => Expr::IndexAccess {
                array: array.clone(),
                index: index.clone(),
            },
            Expr::Word(w) => Expr::Word(w.clone()),
            Expr::Phrase(v) => Expr::Phrase(v.clone()),
            Expr::PropertyAccess { owner, property } => Expr::PropertyAccess {
                owner: owner.clone(),
                property: property.clone(),
            },
            Expr::Call { verb, arguments } => Expr::Call {
                verb: verb.clone(),
                arguments: arguments.clone(),
            },
            Expr::Binding { name, value } => Expr::Binding {
                name: name.clone(),
                value: value.clone(),
            },
            Expr::BinOp { left, op, right } => Expr::BinOp {
                left: left.clone(),
                op: *op,
                right: right.clone(),
            },
            Expr::UnaryOp { op, operand } => Expr::UnaryOp {
                op: *op,
                operand: operand.clone(),
            },
            Expr::Block(stmts) => Expr::Block(stmts.clone()),
        })
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        // Prevent stack overflow on deep equality checks
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match (self, other) {
            (Expr::StringLiteral(a), Expr::StringLiteral(b)) => a == b,
            (Expr::NumberLiteral(a), Expr::NumberLiteral(b)) => a == b,
            (Expr::BooleanLiteral(a), Expr::BooleanLiteral(b)) => a == b,
            (Expr::ArrayLiteral(a), Expr::ArrayLiteral(b)) => a == b,
            (
                Expr::IndexAccess {
                    array: a1,
                    index: i1,
                },
                Expr::IndexAccess {
                    array: a2,
                    index: i2,
                },
            ) => a1 == a2 && i1 == i2,
            (Expr::Word(a), Expr::Word(b)) => a == b,
            (Expr::Phrase(a), Expr::Phrase(b)) => a == b,
            (
                Expr::PropertyAccess {
                    owner: o1,
                    property: p1,
                },
                Expr::PropertyAccess {
                    owner: o2,
                    property: p2,
                },
            ) => o1 == o2 && p1 == p2,
            (
                Expr::Call {
                    verb: v1,
                    arguments: a1,
                },
                Expr::Call {
                    verb: v2,
                    arguments: a2,
                },
            ) => v1 == v2 && a1 == a2,
            (
                Expr::Binding {
                    name: n1,
                    value: v1,
                },
                Expr::Binding {
                    name: n2,
                    value: v2,
                },
            ) => n1 == n2 && v1 == v2,
            (
                Expr::BinOp {
                    left: l1,
                    op: o1,
                    right: r1,
                },
                Expr::BinOp {
                    left: l2,
                    op: o2,
                    right: r2,
                },
            ) => l1 == l2 && o1 == o2 && r1 == r2,
            (
                Expr::UnaryOp {
                    op: o1,
                    operand: op1,
                },
                Expr::UnaryOp {
                    op: o2,
                    operand: op2,
                },
            ) => o1 == o2 && op1 == op2,
            (Expr::Block(a), Expr::Block(b)) => a == b,
            _ => false,
        })
    }
}

impl Drop for Expr {
    fn drop(&mut self) {
        // Prevent stack overflow on deep dropping
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            // Check if we need to drop deep children
            match self {
                Expr::StringLiteral(_)
                | Expr::NumberLiteral(_)
                | Expr::BooleanLiteral(_)
                | Expr::Word(_) => return,
                _ => {}
            }

            // Instead of using unsafe and ManuallyDrop, we take the inner fields
            // of the enum variants and replace them with trivial dummy values.
            // These extracted values are dropped here inside the `maybe_grow` closure,
            // preventing a stack overflow. When `Drop::drop` returns, the compiler
            // will drop the `self` which now only contains trivial values that don't
            // trigger further recursion.
            match self {
                Expr::Phrase(v) => {
                    let _ = std::mem::take(v);
                }
                Expr::Block(v) => {
                    let _ = std::mem::take(v);
                }
                Expr::ArrayLiteral(v) => {
                    let _ = std::mem::take(v);
                }
                Expr::IndexAccess { array, index } => {
                    let _ = std::mem::replace(&mut **array, Expr::BooleanLiteral(false));
                    let _ = std::mem::replace(&mut **index, Expr::BooleanLiteral(false));
                }
                Expr::PropertyAccess { owner, property } => {
                    let _ = std::mem::replace(&mut **owner, Expr::BooleanLiteral(false));
                    let _ = std::mem::replace(&mut **property, Expr::BooleanLiteral(false));
                }
                Expr::Call { verb: _, arguments } => {
                    // verb is a Word, which contains SmolStr. It's safe to let it drop normally,
                    // but since Word doesn't contain Expr, it doesn't recurse.
                    let _ = std::mem::take(arguments);
                }
                Expr::Binding { name: _, value } => {
                    // name is a Word. No recursion.
                    let _ = std::mem::replace(&mut **value, Expr::BooleanLiteral(false));
                }
                Expr::BinOp { left, op: _, right } => {
                    let _ = std::mem::replace(&mut **left, Expr::BooleanLiteral(false));
                    let _ = std::mem::replace(&mut **right, Expr::BooleanLiteral(false));
                }
                Expr::UnaryOp { op: _, operand } => {
                    let _ = std::mem::replace(&mut **operand, Expr::BooleanLiteral(false));
                }
                // Trivial cases
                _ => {}
            }
        });
    }
}

/// Binary operators in GLOSSA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOperator {
    // Arithmetic
    /// Addition (sum) - `ἄθροισμα`
    Add,
    /// Subtraction (difference) - `διαφορά`
    Sub,
    /// Multiplication (product) - `γινόμενον`
    Mul,
    /// Division (quotient/part) - `μέρος`
    Div,
    /// Modulo (remainder) - `ὑπόλοιπον`
    Mod,

    // Comparison
    /// Equality - `ἴσον`
    Eq,
    /// Inequality - `ἄνισον`
    Ne,
    /// Less than - `ἔλαττον`
    Lt,
    /// Less than or equal - `ἔλαττον ἢ ἴσον`
    Le,
    /// Greater than - `μεῖζον`
    Gt,
    /// Greater than or equal - `μεῖζον ἢ ἴσον`
    Ge,

    // Boolean
    /// Logical AND - `καί`
    And,
    /// Logical OR - `ἤ`
    Or,
}

/// Unary operators in GLOSSA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    /// Logical Negation - `οὐ`, `οὐκ`, `οὐχ`
    Not,
    /// Arithmetic Negation
    Neg,
    /// Unwrap (Extract value) - `!` suffix
    Unwrap,
}

/// Represents a parsed word from the ΓΛΩΣΣΑ source code.
///
/// A `Word` in the AST is not just a `String`. Because Ancient Greek has
/// rich diacritics (accents, breathings) and capitalization, comparing words
/// directly is error-prone. This struct acts as a bridge between the
/// human-readable source and the compiler's internal matching logic,
/// preserving the original polytonic text (for display) while providing
/// a normalized version for compiler analysis.
///
/// ## Why it exists
///
/// In the morphological analysis phase, the compiler needs to look up words
/// in its lexicon. If a user writes `Ἄνθρωπος` (Capital A, smooth breathing, acute accent),
/// the compiler must recognize it as the same lemma as `ἄνθρωπος` (lowercase a, smooth, acute)
/// or even `ανθρωπος` (stripped of all diacritics).
///
/// Storing both the `original` string (for exact error reporting) and the `normalized`
/// string (for lexicon lookups and comparisons) in a single struct solves this problem
/// at the parsing stage, preventing expensive re-allocations later.
///
/// ## Examples
///
/// You can construct a `Word` via `Word::new` for convenience, or construct it directly
/// using [`smol_str::SmolStr`] to avoid heap allocations for small strings.
///
/// ```rust
/// use glossa::ast::Word;
///
/// let word = Word::new("Ἀθῆναι");
/// assert_eq!(word.original.as_str(), "Ἀθῆναι");
/// assert_eq!(word.normalized.as_str(), "αθηναι");
/// ```
///
/// ```rust
/// use glossa::ast::Word;
/// use smol_str::SmolStr;
///
/// // Create a word representing the noun "ἄνθρωπος" (man) directly
/// let man = Word {
///     original: SmolStr::new("Ἄνθρωπος"),
///     normalized: SmolStr::new("ανθρωπος"),
/// };
///
/// assert_eq!(man.original.as_str(), "Ἄνθρωπος");
/// assert_eq!(man.normalized.as_str(), "ανθρωπος");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    /// The exact string from the source code, preserving case and diacritics.
    /// Useful for accurate compilation error messages and source maps.
    pub original: SmolStr,
    /// The lowercase, diacritic-stripped version of the word.
    /// Used for lexicon lookups, keyword matching, and semantic hashing.
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
