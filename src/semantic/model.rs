//! The Semantic Model (τὸ Σημασιολογικὸν Ἔνδυμα)
//!
//! This module serves as the pure data container for the analyzed program.
//! It separates the *state* (the Abstract Syntax Tree enriched with type and scope information)
//! from the *behavior* ([`crate::semantic::analyzer`]) and the *type system* ([`crate::semantic::types`]).
//!
//! # Architectural Purpose: The "Atlas" Pattern
//!
//! By isolating the data structures here, we prevent circular dependencies between:
//! * `analyzer.rs` (needs `model.rs` and `types.rs`)
//! * `types.rs` (needs `model.rs` for function signatures)
//! * `assembly.rs` (needs `model.rs` to build statements)
//!
//! Think of this module as the "dictionary" that all other semantic phases read from and write to.
//!
//! # The Journey of a Statement
//!
//! 1. Starts as raw text: `«χαῖρε» λέγε.`
//! 2. Parsed into [`crate::ast::Statement`].
//! 3. Assembled into [`crate::semantic::assembly::AssembledStatement`].
//! 4. Finally transformed into an [`AnalyzedStatement`] defined here.
//!
//! # Examples
//!
//! ```rust
//! use glossa::semantic::{AnalyzedStatement, AnalyzedExpr, AnalyzedExprKind};
//! use glossa::semantic::GlossaType;
//! use smol_str::SmolStr;
//!
//! // Creating an analyzed literal: `5`
//! let five_expr = AnalyzedExpr {
//!     expr: AnalyzedExprKind::NumberLiteral(5),
//!     glossa_type: GlossaType::Number,
//! };
//!
//! // Creating an analyzed binding: `ξ 5 ἔστω.` -> `let g_x = 5;`
//! let binding = AnalyzedStatement::Binding {
//!     name: SmolStr::new("ξ"),
//!     value: five_expr,
//!     mutable: false,
//! };
//!
//! match binding {
//!     AnalyzedStatement::Binding { name, .. } => assert_eq!(name, "ξ"),
//!     _ => unreachable!(),
//! }
//! ```

use crate::semantic::types::GlossaType;
use smol_str::SmolStr;

/// Analyzed statement
///
/// Represents a statement after semantic analysis, where names are resolved,
/// types are inferred, and word order is normalized.
#[derive(Debug, Clone)]
pub enum AnalyzedStatement {
    /// Variable binding
    ///
    /// # Example
    /// ```glossa
    /// ξ πέντε ἔστω.
    /// ```
    /// -> `let g_x = 5;`
    Binding {
        name: SmolStr,
        value: AnalyzedExpr,
        mutable: bool,
    },
    /// Assignment to existing variable
    ///
    /// # Example
    /// ```glossa
    /// ξ δέκα γίγνεται.
    /// ```
    /// -> `g_x = 10;`
    Assignment { name: SmolStr, value: AnalyzedExpr },
    /// Print statement
    ///
    /// # Example
    /// ```glossa
    /// «χαῖρε» λέγε.
    /// ```
    /// -> `println!("χαῖρε");`
    Print(Vec<AnalyzedExpr>),
    /// Expression statement (side effect)
    ///
    /// # Example
    /// ```glossa
    /// array.push(1).
    /// ```
    Expression(Vec<AnalyzedExpr>),
    /// Query statement (print with newline)
    ///
    /// # Example
    /// ```glossa
    /// ξ?
    /// ```
    /// -> `println!("{}", g_x);`
    Query(Vec<AnalyzedExpr>),
    /// If conditional
    ///
    /// # Example
    /// ```glossa
    /// εἰ ξ > 5, ... εἰ δὲ μή, ...
    /// ```
    /// -> `if g_x > 5 { ... } else { ... }`
    If {
        condition: Box<AnalyzedExpr>,
        then_body: Vec<AnalyzedStatement>,
        else_body: Option<Vec<AnalyzedStatement>>,
    },
    /// While loop
    ///
    /// # Example
    /// ```glossa
    /// ἕως ξ < 10, ...
    /// ```
    /// -> `while g_x < 10 { ... }`
    While {
        condition: Box<AnalyzedExpr>,
        body: Vec<AnalyzedStatement>,
    },
    /// For loop
    ///
    /// # Example
    /// ```glossa
    /// διὰ α, β λέγε.
    /// ```
    /// -> `for b in a { println!("{}", b); }`
    For {
        variable: SmolStr,
        iterator: Box<AnalyzedExpr>,
        body: Vec<AnalyzedStatement>,
    },
    /// Match expression
    ///
    /// # Example
    /// ```glossa
    /// κατά ξ { 1 => ... }
    /// ```
    /// -> `match g_x { 1 => ... }`
    Match {
        scrutinee: Box<AnalyzedExpr>,
        arms: Vec<(AnalyzedExpr, Vec<AnalyzedStatement>)>,
    },
    /// Break statement
    ///
    /// # Example
    /// ```glossa
    /// παῦε.
    /// ```
    /// -> `break;`
    Break,
    /// Continue statement
    ///
    /// # Example
    /// ```glossa
    /// συνέχιζε.
    /// ```
    /// -> `continue;`
    Continue,
    /// Return statement
    ///
    /// # Example
    /// ```glossa
    /// δός 5.
    /// ```
    /// -> `return 5;`
    Return { value: Option<Box<AnalyzedExpr>> },
    /// Function definition
    ///
    /// # Example
    /// ```glossa
    /// add ὁρίζειν (a, b)· ...
    /// ```
    /// -> `fn g_add(a: i64, b: i64) { ... }`
    FunctionDef {
        name: SmolStr,
        params: Vec<(SmolStr, Option<GlossaType>)>,
        body: Vec<AnalyzedStatement>,
        return_type: Option<GlossaType>,
    },
    /// Type definition (struct)
    ///
    /// # Example
    /// ```glossa
    /// εἶδος User ...
    /// ```
    /// -> `struct User { ... }`
    TypeDefinition {
        name: SmolStr,
        fields: Vec<(SmolStr, GlossaType)>,
    },
    /// Trait definition
    ///
    /// # Example
    /// ```glossa
    /// χαρακτήρ Show ...
    /// ```
    /// -> `trait Show { ... }`
    TraitDefinition {
        name: SmolStr,
        methods: Vec<AnalyzedMethod>,
    },
    /// Trait implementation
    ///
    /// # Example
    /// ```glossa
    /// εἶδος User τῷ Show ἐμπίπτειν ...
    /// ```
    /// -> `impl Show for User { ... }`
    TraitImplementation {
        trait_name: SmolStr,
        type_name: SmolStr,
        methods: Vec<AnalyzedMethod>,
    },
    /// Test declaration
    ///
    /// # Example
    /// ```glossa
    /// δοκιμή «test» ... τέλος.
    /// ```
    /// -> `#[test] fn test() { ... }`
    TestDeclaration {
        name: String,
        body: Vec<AnalyzedStatement>,
    },
}

/// An analyzed method (used in traits and implementations)
#[derive(Debug, Clone)]
pub struct AnalyzedMethod {
    pub name: SmolStr,
    pub params: Vec<(SmolStr, GlossaType)>,
    pub body: Option<Vec<AnalyzedStatement>>, // Some for default/impl methods, None for required
    pub return_type: Option<GlossaType>,
}

/// Analyzed expression with type information
#[derive(Debug, Clone)]
pub struct AnalyzedExpr {
    pub expr: AnalyzedExprKind,
    pub glossa_type: GlossaType,
}

/// Kind of analyzed expression
#[derive(Debug, Clone)]
pub enum AnalyzedExprKind {
    /// String literal
    ///
    /// # Example
    /// `«hello»` -> `StringLiteral("hello")`
    StringLiteral(String),

    /// Number literal (integer)
    ///
    /// # Example
    /// `42` -> `NumberLiteral(42)`
    NumberLiteral(i64),

    /// Boolean literal
    ///
    /// # Example
    /// `ἀληθές` -> `BooleanLiteral(true)`
    BooleanLiteral(bool),

    /// Variable reference
    ///
    /// # Example
    /// `x` -> `Variable("x")`
    Variable(SmolStr),

    /// Property access (field access)
    ///
    /// # Example
    /// `user.name` -> `PropertyAccess { owner: user, property: "name" }`
    PropertyAccess {
        owner: Box<AnalyzedExpr>,
        property: SmolStr,
    },

    /// Verb call (function call using verb syntax)
    ///
    /// # Example
    /// `λέγει` (says) -> `VerbCall { verb: "say", args: [] }`
    VerbCall {
        verb: SmolStr,
        args: Vec<AnalyzedExpr>,
    },

    /// Binary operation (arithmetic, comparison, boolean)
    ///
    /// # Example
    /// `1 + 2` -> `BinOp { left: 1, op: Add, right: 2 }`
    BinOp {
        left: Box<AnalyzedExpr>,
        op: crate::morphology::lexicon::BinaryOp,
        right: Box<AnalyzedExpr>,
    },

    /// Unary operation (negation)
    ///
    /// # Example
    /// `οὐκ x` -> `UnaryOp { op: Not, operand: x }`
    UnaryOp {
        op: crate::morphology::lexicon::UnaryOp,
        operand: Box<AnalyzedExpr>,
    },

    /// Range expression for loops (start..end or start..=end)
    ///
    /// # Example
    /// `1..10`
    Range {
        start: Box<AnalyzedExpr>,
        end: Box<AnalyzedExpr>,
        inclusive: bool,
    },

    /// Array literal [1, 2, 3]
    ///
    /// # Example
    /// `[1, 2, 3]`
    ArrayLiteral(Vec<AnalyzedExpr>),

    /// Some(value) - `Option<T>` constructor
    ///
    /// # Example
    /// `τί` -> `Some`
    Some(Box<AnalyzedExpr>),

    /// None - `Option<T>` empty value
    ///
    /// # Example
    /// `οὐδέν` -> `None`
    None,

    /// Ok(value) - `Result<T,E>` success constructor
    ///
    /// # Example
    /// `ἐπιτυχία` -> `Ok`
    Ok(Box<AnalyzedExpr>),

    /// Err(error) - `Result<T,E>` error constructor
    ///
    /// # Example
    /// `σφάλμα` -> `Err`
    Err(Box<AnalyzedExpr>),

    /// Unwrap operator (!) - confident extraction from `Option`/`Result`
    ///
    /// # Example
    /// `x!` -> `x.unwrap()`
    Unwrap(Box<AnalyzedExpr>),

    /// Try operator (`;` after `Option`/`Result`) - propagates None/Err upward
    ///
    /// # Example
    /// `x;` -> `x?`
    Try(Box<AnalyzedExpr>),

    /// Index access `array[index]`
    ///
    /// # Example
    /// `arr[0]`
    IndexAccess {
        array: Box<AnalyzedExpr>,
        index: Box<AnalyzedExpr>,
    },

    /// Function call to user-defined function
    ///
    /// # Example
    /// `my_func(arg)`
    FunctionCall {
        func: SmolStr,
        args: Vec<AnalyzedExpr>,
    },

    /// Method call receiver.method(args)
    ///
    /// # Example
    /// `vec.push(1)`
    MethodCall {
        receiver: Box<AnalyzedExpr>,
        method: SmolStr,
        args: Vec<AnalyzedExpr>,
    },

    /// Trait method call `receiver.<TraitName>::method(args)` (from trait impl)
    ///
    /// # Example
    /// `user.Show::print()`
    TraitMethodCall {
        receiver: Box<AnalyzedExpr>,
        trait_name: SmolStr,
        method_name: SmolStr,
        args: Vec<AnalyzedExpr>,
    },

    /// Struct instantiation: `variable νέον type_name args... ἔστω`
    ///
    /// # Example
    /// `x new User "name" 42`
    StructInstantiation {
        type_name: SmolStr,
        fields: Vec<SmolStr>, // Field names from struct definition
        args: Vec<AnalyzedExpr>,
    },

    /// Lambda/closure |params| body
    ///
    /// # Example
    /// `|x| x + 1`
    Lambda {
        params: Vec<SmolStr>,
        body: Box<AnalyzedExpr>,
        capture_mode: CaptureMode,
    },

    /// Collection constructor (HashSet::new(), HashMap::new())
    CollectionNew { collection_type: String },

    /// Boolean assertion: δεῖ (condition must be true)
    Assert { condition: Box<AnalyzedExpr> },

    /// Equality assertion: ἰσοῦται (values must be equal)
    AssertEq {
        left: Box<AnalyzedExpr>,
        right: Box<AnalyzedExpr>,
    },
}

/// Capture mode for closures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    /// Borrow captured variables (default for present participles)
    ///
    /// Used when the lambda is executed immediately (e.g. `map`, `filter`).
    Borrow,

    /// Move captured variables (for aorist participles)
    ///
    /// Used when the lambda might outlive the current scope or needs ownership.
    Move,

    /// Memoize result (for perfect participles)
    ///
    /// Used to cache the result of the lambda for identical inputs.
    /// This turns the closure into a lazy, memoized value.
    Memoize,
}

// --- Trait and Type Definitions (moved from types.rs) ---

/// Trait definition for semantic analysis
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: SmolStr,
    pub methods: Vec<AnalyzedMethod>,
}
