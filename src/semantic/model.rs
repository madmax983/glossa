//! Semantic AST and Data Models
//!
//! This module contains the data structures that represent the analyzed program.
//! It decouples the data from the logic to prevent circular dependencies.

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
        /// The identifier assigned to the memory slot.
        name: SmolStr,
        /// The fully evaluated expression filling the memory slot.
        value: AnalyzedExpr,
        /// Whether this value is allowed to mutate or is locked.
        mutable: bool,
    },
    /// Assignment to existing variable
    ///
    /// # Example
    /// ```glossa
    /// ξ δέκα γίγνεται.
    /// ```
    /// -> `g_x = 10;`
    Assignment {
        /// The identifier representing the memory destination.
        name: SmolStr,
        /// The new value resolving into the target.
        value: AnalyzedExpr,
    },
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
        /// The boolean expression acting as the gatekeeper.
        condition: Box<AnalyzedExpr>,
        /// The sequence of operations executed if the gate is opened.
        then_body: Vec<AnalyzedStatement>,
        /// The optional sequence of operations executed if the gate is closed.
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
        /// The condition that must remain true
        condition: Box<AnalyzedExpr>,
        /// The list of statements comprising the loop body
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
        /// The identifier bound to the current element in the traversal.
        variable: SmolStr,
        /// The collection being sequentially traversed.
        iterator: Box<AnalyzedExpr>,
        /// The block executed for each element.
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
        /// The value being interrogated against the arms.
        scrutinee: Box<AnalyzedExpr>,
        /// The potential structural destructuring arms.
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
    Return {
        /// The computed outcome carried back to the caller.
        value: Option<Box<AnalyzedExpr>>,
    },
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
    /// The specific kind of expression (e.g., literal, operation)
    pub expr: AnalyzedExprKind,
    /// The resolved type of the expression
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
        /// The expression whose property is being accessed
        owner: Box<AnalyzedExpr>,
        /// The name of the property being accessed
        property: SmolStr,
    },

    /// Verb call (function call using verb syntax)
    ///
    /// # Example
    /// `λέγει` (says) -> `VerbCall { verb: "say", args: [] }`
    VerbCall {
        /// The name of the verb being called
        verb: SmolStr,
        /// The arguments passed to the verb
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
        /// The starting value of the range
        start: Box<AnalyzedExpr>,
        /// The ending value of the range
        end: Box<AnalyzedExpr>,
        /// Whether the range includes its end value
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
        /// The array expression being indexed
        array: Box<AnalyzedExpr>,
        /// The index expression
        index: Box<AnalyzedExpr>,
    },

    /// Function call to user-defined function
    ///
    /// # Example
    /// `my_func(arg)`
    FunctionCall {
        /// The name of the function to call
        func: SmolStr,
        /// The arguments passed to the function
        args: Vec<AnalyzedExpr>,
    },

    /// Method call receiver.method(args)
    ///
    /// # Example
    /// `vec.push(1)`
    MethodCall {
        /// The expression whose method is being called
        receiver: Box<AnalyzedExpr>,
        /// The name of the method
        method: SmolStr,
        /// The arguments passed to the method
        args: Vec<AnalyzedExpr>,
    },

    /// Trait method call `receiver.<TraitName>::method(args)` (from trait impl)
    ///
    /// # Example
    /// `user.Show::print()`
    TraitMethodCall {
        /// The expression implementing the trait
        receiver: Box<AnalyzedExpr>,
        /// The name of the trait
        trait_name: SmolStr,
        /// The name of the trait method
        method_name: SmolStr,
        /// The arguments passed to the method
        args: Vec<AnalyzedExpr>,
    },

    /// Struct instantiation: `variable νέον type_name args... ἔστω`
    ///
    /// # Example
    /// `x new User "name" 42`
    StructInstantiation {
        /// The name of the struct type
        type_name: SmolStr,
        /// The names of the fields in the struct definition
        fields: Vec<SmolStr>, // Field names from struct definition
        /// The arguments corresponding to each field
        args: Vec<AnalyzedExpr>,
    },

    /// Lambda/closure |params| body
    ///
    /// # Example
    /// `|x| x + 1`
    Lambda {
        /// The parameter names for the closure
        params: Vec<SmolStr>,
        /// The expression body of the closure
        body: Box<AnalyzedExpr>,
        /// The capture mode defining how environment variables are handled
        capture_mode: CaptureMode,
    },

    /// Collection constructor (HashSet::new(), HashMap::new())
    CollectionNew {
        /// The underlying collection type (e.g., "HashSet")
        collection_type: String,
    },

    /// Boolean assertion: δεῖ (condition must be true)
    Assert {
        /// The condition that must evaluate to true
        condition: Box<AnalyzedExpr>,
    },

    /// Equality assertion: ἰσοῦται (values must be equal)
    AssertEq {
        /// The left-hand side value
        left: Box<AnalyzedExpr>,
        /// The right-hand side value
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
    /// The name of the trait
    pub name: SmolStr,
    /// The list of methods defined in the trait
    pub methods: Vec<AnalyzedMethod>,
}

/// Trait implementation for a type
#[derive(Debug, Clone)]
pub struct TraitImpl {
    /// The name of the trait being implemented
    pub trait_name: SmolStr,
    /// The name of the type implementing the trait
    pub type_name: SmolStr,
}
