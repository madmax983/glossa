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
    /// `ξ πέντε ἔστω.` -> `let g_x = 5;`
    Binding {
        name: SmolStr,
        value: AnalyzedExpr,
        mutable: bool,
    },
    /// Assignment to existing variable
    ///
    /// # Example
    /// `ξ δέκα γίγνεται.` -> `g_x = 10;`
    Assignment { name: SmolStr, value: AnalyzedExpr },
    /// Print statement
    ///
    /// # Example
    /// `«χαῖρε» λέγε.` -> `println!("χαῖρε");`
    Print(Vec<AnalyzedExpr>),
    /// Expression statement (side effect)
    ///
    /// # Example
    /// `array.push(1).`
    Expression(Vec<AnalyzedExpr>),
    /// Query statement (print with newline)
    ///
    /// # Example
    /// `ξ?` -> `println!("{}", g_x);`
    Query(Vec<AnalyzedExpr>),
    /// If conditional
    ///
    /// # Example
    /// `εἰ ξ > 5, ... εἰ δὲ μή, ...`
    If {
        condition: Box<AnalyzedExpr>,
        then_body: Vec<AnalyzedStatement>,
        else_body: Option<Vec<AnalyzedStatement>>,
    },
    /// While loop
    ///
    /// # Example
    /// `ἕως ξ < 10, ...`
    While {
        condition: Box<AnalyzedExpr>,
        body: Vec<AnalyzedStatement>,
    },
    /// For loop
    ///
    /// # Example
    /// `διὰ α, β λέγε.` -> `for b in a { println!("{}", b); }`
    For {
        variable: SmolStr,
        iterator: Box<AnalyzedExpr>,
        body: Vec<AnalyzedStatement>,
    },
    /// Match expression
    ///
    /// # Example
    /// `κατά ξ { 1 => ... }`
    Match {
        scrutinee: Box<AnalyzedExpr>,
        arms: Vec<(AnalyzedExpr, Vec<AnalyzedStatement>)>,
    },
    /// Break statement
    ///
    /// # Example
    /// `παῦε.`
    Break,
    /// Continue statement
    ///
    /// # Example
    /// `συνέχιζε.`
    Continue,
    /// Return statement
    ///
    /// # Example
    /// `δός 5.`
    Return { value: Option<Box<AnalyzedExpr>> },
    /// Function definition
    ///
    /// # Example
    /// `func ὁρίζειν (x)· ...`
    FunctionDef {
        name: SmolStr,
        params: Vec<(SmolStr, Option<GlossaType>)>,
        body: Vec<AnalyzedStatement>,
        return_type: Option<GlossaType>,
    },
    /// Type definition (struct)
    ///
    /// # Example
    /// `εἶδος User ...`
    TypeDefinition {
        name: SmolStr,
        fields: Vec<(SmolStr, GlossaType)>,
    },
    /// Trait definition
    ///
    /// # Example
    /// `χαρακτήρ Show ...`
    TraitDefinition {
        name: SmolStr,
        methods: Vec<AnalyzedMethod>,
    },
    /// Trait implementation
    ///
    /// # Example
    /// `εἶδος User τῷ Show ἐμπίπτειν ...`
    TraitImplementation {
        trait_name: SmolStr,
        type_name: SmolStr,
        methods: Vec<AnalyzedMethod>,
    },
    /// Test declaration
    ///
    /// # Example
    /// `δοκιμή «test» ... τέλος.`
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
    StringLiteral(String),
    /// Number literal (integer)
    NumberLiteral(i64),
    /// Boolean literal
    BooleanLiteral(bool),
    /// Variable reference
    Variable(SmolStr),
    /// Property access (field access)
    PropertyAccess {
        owner: Box<AnalyzedExpr>,
        property: SmolStr,
    },
    /// Verb call (function call using verb syntax)
    VerbCall {
        verb: SmolStr,
        args: Vec<AnalyzedExpr>,
    },
    /// Binary operation (arithmetic, comparison, boolean)
    BinOp {
        left: Box<AnalyzedExpr>,
        op: crate::morphology::lexicon::BinaryOp,
        right: Box<AnalyzedExpr>,
    },
    /// Unary operation (negation)
    UnaryOp {
        op: crate::morphology::lexicon::UnaryOp,
        operand: Box<AnalyzedExpr>,
    },
    /// Range expression for loops (start..end or start..=end)
    Range {
        start: Box<AnalyzedExpr>,
        end: Box<AnalyzedExpr>,
        inclusive: bool,
    },
    /// Array literal [1, 2, 3]
    ArrayLiteral(Vec<AnalyzedExpr>),
    /// Some(value) - `Option<T>` constructor
    Some(Box<AnalyzedExpr>),
    /// None - `Option<T>` empty value
    None,
    /// Ok(value) - `Result<T,E>` success constructor
    Ok(Box<AnalyzedExpr>),
    /// Err(error) - `Result<T,E>` error constructor
    Err(Box<AnalyzedExpr>),
    /// Unwrap operator (!) - confident extraction from `Option`/`Result`
    Unwrap(Box<AnalyzedExpr>),
    /// Try operator (`;` after `Option`/`Result`) - propagates None/Err upward
    Try(Box<AnalyzedExpr>),
    /// Index access `array[index]`
    IndexAccess {
        array: Box<AnalyzedExpr>,
        index: Box<AnalyzedExpr>,
    },
    /// Function call to user-defined function
    FunctionCall {
        func: SmolStr,
        args: Vec<AnalyzedExpr>,
    },
    /// Method call receiver.method(args)
    MethodCall {
        receiver: Box<AnalyzedExpr>,
        method: SmolStr,
        args: Vec<AnalyzedExpr>,
    },
    /// Trait method call `receiver.<TraitName>::method(args)` (from trait impl)
    TraitMethodCall {
        receiver: Box<AnalyzedExpr>,
        trait_name: SmolStr,
        method_name: SmolStr,
        args: Vec<AnalyzedExpr>,
    },
    /// Struct instantiation: `variable νέον type_name args... ἔστω`
    StructInstantiation {
        type_name: SmolStr,
        fields: Vec<SmolStr>, // Field names from struct definition
        args: Vec<AnalyzedExpr>,
    },
    /// Lambda/closure |params| body
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
    Borrow,

    /// Move captured variables (for aorist participles)
    Move,

    /// Memoize result (for perfect participles)
    Memoize,
}

// --- Trait and Type Definitions (moved from types.rs) ---

/// Trait definition for semantic analysis
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: SmolStr,
    pub methods: Vec<AnalyzedMethod>,
}

/// Trait implementation for a type
#[derive(Debug, Clone)]
pub struct TraitImpl {
    pub trait_name: SmolStr,
    pub type_name: SmolStr,
}
