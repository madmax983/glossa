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
#[derive(Clone)]
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

impl std::fmt::Debug for AnalyzedStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            AnalyzedStatement::Binding {
                name,
                value,
                mutable,
            } => f
                .debug_struct("Binding")
                .field("name", name)
                .field("value", value)
                .field("mutable", mutable)
                .finish(),
            AnalyzedStatement::Assignment { name, value } => f
                .debug_struct("Assignment")
                .field("name", name)
                .field("value", value)
                .finish(),
            AnalyzedStatement::Print(exprs) => f.debug_tuple("Print").field(exprs).finish(),
            AnalyzedStatement::Expression(exprs) => {
                f.debug_tuple("Expression").field(exprs).finish()
            }
            AnalyzedStatement::Query(exprs) => f.debug_tuple("Query").field(exprs).finish(),
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => f
                .debug_struct("If")
                .field("condition", condition)
                .field("then_body", then_body)
                .field("else_body", else_body)
                .finish(),
            AnalyzedStatement::While { condition, body } => f
                .debug_struct("While")
                .field("condition", condition)
                .field("body", body)
                .finish(),
            AnalyzedStatement::For {
                variable,
                iterator,
                body,
            } => f
                .debug_struct("For")
                .field("variable", variable)
                .field("iterator", iterator)
                .field("body", body)
                .finish(),
            AnalyzedStatement::Match { scrutinee, arms } => f
                .debug_struct("Match")
                .field("scrutinee", scrutinee)
                .field("arms", arms)
                .finish(),
            AnalyzedStatement::Break => f.debug_tuple("Break").finish(),
            AnalyzedStatement::Continue => f.debug_tuple("Continue").finish(),
            AnalyzedStatement::Return { value } => {
                f.debug_struct("Return").field("value", value).finish()
            }
            AnalyzedStatement::FunctionDef {
                name,
                params,
                body,
                return_type,
            } => f
                .debug_struct("FunctionDef")
                .field("name", name)
                .field("params", params)
                .field("body", body)
                .field("return_type", return_type)
                .finish(),
            AnalyzedStatement::TypeDefinition { name, fields } => f
                .debug_struct("TypeDefinition")
                .field("name", name)
                .field("fields", fields)
                .finish(),
            AnalyzedStatement::TraitDefinition { name, methods } => f
                .debug_struct("TraitDefinition")
                .field("name", name)
                .field("methods", methods)
                .finish(),
            AnalyzedStatement::TraitImplementation {
                trait_name,
                type_name,
                methods,
            } => f
                .debug_struct("TraitImplementation")
                .field("trait_name", trait_name)
                .field("type_name", type_name)
                .field("methods", methods)
                .finish(),
            AnalyzedStatement::TestDeclaration { name, body } => f
                .debug_struct("TestDeclaration")
                .field("name", name)
                .field("body", body)
                .finish(),
        })
    }
}

/// An analyzed method (used in traits and implementations)
#[derive(Clone)]
pub struct AnalyzedMethod {
    pub name: SmolStr,
    pub params: Vec<(SmolStr, GlossaType)>,
    pub body: Option<Vec<AnalyzedStatement>>, // Some for default/impl methods, None for required
    pub return_type: Option<GlossaType>,
}

impl std::fmt::Debug for AnalyzedMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            f.debug_struct("AnalyzedMethod")
                .field("name", &self.name)
                .field("params", &self.params)
                .field("body", &self.body)
                .field("return_type", &self.return_type)
                .finish()
        })
    }
}

/// Analyzed expression with type information
#[derive(Clone)]
pub struct AnalyzedExpr {
    pub expr: AnalyzedExprKind,
    pub glossa_type: GlossaType,
}

impl std::fmt::Debug for AnalyzedExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            f.debug_struct("AnalyzedExpr")
                .field("expr", &self.expr)
                .field("glossa_type", &self.glossa_type)
                .finish()
        })
    }
}

/// Kind of analyzed expression
#[derive(Clone)]
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

impl std::fmt::Debug for AnalyzedExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            AnalyzedExprKind::StringLiteral(s) => f.debug_tuple("StringLiteral").field(s).finish(),
            AnalyzedExprKind::NumberLiteral(n) => f.debug_tuple("NumberLiteral").field(n).finish(),
            AnalyzedExprKind::BooleanLiteral(b) => {
                f.debug_tuple("BooleanLiteral").field(b).finish()
            }
            AnalyzedExprKind::Variable(v) => f.debug_tuple("Variable").field(v).finish(),
            AnalyzedExprKind::PropertyAccess { owner, property } => f
                .debug_struct("PropertyAccess")
                .field("owner", owner)
                .field("property", property)
                .finish(),
            AnalyzedExprKind::VerbCall { verb, args } => f
                .debug_struct("VerbCall")
                .field("verb", verb)
                .field("args", args)
                .finish(),
            AnalyzedExprKind::BinOp { left, op, right } => f
                .debug_struct("BinOp")
                .field("left", left)
                .field("op", op)
                .field("right", right)
                .finish(),
            AnalyzedExprKind::UnaryOp { op, operand } => f
                .debug_struct("UnaryOp")
                .field("op", op)
                .field("operand", operand)
                .finish(),
            AnalyzedExprKind::Range {
                start,
                end,
                inclusive,
            } => f
                .debug_struct("Range")
                .field("start", start)
                .field("end", end)
                .field("inclusive", inclusive)
                .finish(),
            AnalyzedExprKind::ArrayLiteral(v) => f.debug_tuple("ArrayLiteral").field(v).finish(),
            AnalyzedExprKind::Some(v) => f.debug_tuple("Some").field(v).finish(),
            AnalyzedExprKind::None => f.debug_tuple("None").finish(),
            AnalyzedExprKind::Ok(v) => f.debug_tuple("Ok").field(v).finish(),
            AnalyzedExprKind::Err(v) => f.debug_tuple("Err").field(v).finish(),
            AnalyzedExprKind::Unwrap(v) => f.debug_tuple("Unwrap").field(v).finish(),
            AnalyzedExprKind::Try(v) => f.debug_tuple("Try").field(v).finish(),
            AnalyzedExprKind::IndexAccess { array, index } => f
                .debug_struct("IndexAccess")
                .field("array", array)
                .field("index", index)
                .finish(),
            AnalyzedExprKind::FunctionCall { func, args } => f
                .debug_struct("FunctionCall")
                .field("func", func)
                .field("args", args)
                .finish(),
            AnalyzedExprKind::MethodCall {
                receiver,
                method,
                args,
            } => f
                .debug_struct("MethodCall")
                .field("receiver", receiver)
                .field("method", method)
                .field("args", args)
                .finish(),
            AnalyzedExprKind::TraitMethodCall {
                receiver,
                trait_name,
                method_name,
                args,
            } => f
                .debug_struct("TraitMethodCall")
                .field("receiver", receiver)
                .field("trait_name", trait_name)
                .field("method_name", method_name)
                .field("args", args)
                .finish(),
            AnalyzedExprKind::StructInstantiation {
                type_name,
                fields,
                args,
            } => f
                .debug_struct("StructInstantiation")
                .field("type_name", type_name)
                .field("fields", fields)
                .field("args", args)
                .finish(),
            AnalyzedExprKind::Lambda {
                params,
                body,
                capture_mode,
            } => f
                .debug_struct("Lambda")
                .field("params", params)
                .field("body", body)
                .field("capture_mode", capture_mode)
                .finish(),
            AnalyzedExprKind::CollectionNew { collection_type } => f
                .debug_struct("CollectionNew")
                .field("collection_type", collection_type)
                .finish(),
            AnalyzedExprKind::Assert { condition } => f
                .debug_struct("Assert")
                .field("condition", condition)
                .finish(),
            AnalyzedExprKind::AssertEq { left, right } => f
                .debug_struct("AssertEq")
                .field("left", left)
                .field("right", right)
                .finish(),
        })
    }
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
#[derive(Clone)]
pub struct TraitDef {
    pub name: SmolStr,
    pub methods: Vec<AnalyzedMethod>,
}

impl std::fmt::Debug for TraitDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            f.debug_struct("TraitDef")
                .field("name", &self.name)
                .field("methods", &self.methods)
                .finish()
        })
    }
}

/// Trait implementation for a type
#[derive(Clone)]
pub struct TraitImpl {
    pub trait_name: SmolStr,
    pub type_name: SmolStr,
}

impl std::fmt::Debug for TraitImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            f.debug_struct("TraitImpl")
                .field("trait_name", &self.trait_name)
                .field("type_name", &self.type_name)
                .finish()
        })
    }
}
