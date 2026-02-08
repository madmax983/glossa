//! Semantic AST and Data Models
//!
//! This module contains the data structures that represent the analyzed program.
//! It decouples the data from the logic to prevent circular dependencies.

use crate::semantic::types::GlossaType;
use smol_str::SmolStr;

/// Analyzed statement
#[derive(Debug, Clone)]
pub struct AnalyzedStatement {
    pub kind: StatementKind,
    pub expressions: Vec<AnalyzedExpr>,
}

/// The kind of statement
#[derive(Debug, Clone)]
pub enum StatementKind {
    /// Variable binding: ξ πέντε ἔστω
    Binding {
        name: SmolStr,
        value_type: GlossaType,
        mutable: bool,
    },
    /// Assignment: ξ δέκα γίγνεται
    Assignment {
        name: SmolStr,
        value_type: GlossaType,
    },
    /// Print statement: «χαῖρε» λέγε
    Print,
    /// Expression statement
    Expression,
    /// Query: ξ?
    Query,
    /// If conditional: εἰ condition, body [εἰ δὲ μή, else_body]
    If {
        condition: Box<AnalyzedExpr>,
        then_body: Vec<AnalyzedStatement>,
        else_body: Option<Vec<AnalyzedStatement>>,
    },
    /// While loop: ἕως condition, body
    While {
        condition: Box<AnalyzedExpr>,
        body: Vec<AnalyzedStatement>,
    },
    /// For loop: διά/ἀπό...μέχρι
    For {
        variable: SmolStr,
        iterator: Box<AnalyzedExpr>,
        body: Vec<AnalyzedStatement>,
    },
    /// Match expression: κατά scrutinee { arms }
    Match {
        scrutinee: Box<AnalyzedExpr>,
        arms: Vec<(AnalyzedExpr, Vec<AnalyzedStatement>)>,
    },
    /// Break: παῦε
    Break,
    /// Continue: συνέχιζε
    Continue,
    /// Return: δός value
    Return { value: Option<Box<AnalyzedExpr>> },
    /// Function definition: name ὁρίζειν params· body
    FunctionDef {
        name: SmolStr,
        params: Vec<(SmolStr, Option<GlossaType>)>,
        body: Vec<AnalyzedStatement>,
        return_type: Option<GlossaType>,
    },
    /// Type definition: εἶδος name ὁρίζειν { fields }
    TypeDefinition {
        name: SmolStr,
        fields: Vec<(SmolStr, GlossaType)>,
    },
    /// Trait definition: χαρακτήρ name ὁρίζειν { methods }
    TraitDefinition {
        name: SmolStr,
        methods: Vec<AnalyzedMethod>,
    },
    /// Trait implementation: εἶδος Type τῷ Trait ἐμπίπτειν { methods }
    TraitImplementation {
        trait_name: SmolStr,
        type_name: SmolStr,
        methods: Vec<AnalyzedMethod>,
    },
    /// Test declaration: δοκιμή «test name» ... τέλος
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
    StringLiteral(String),
    NumberLiteral(i64),
    BooleanLiteral(bool),
    Variable(SmolStr),
    PropertyAccess {
        owner: Box<AnalyzedExpr>,
        property: SmolStr,
    },
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
    CollectionNew {
        collection_type: String,
    },
    /// Boolean assertion: δεῖ (condition must be true)
    Assert {
        condition: Box<AnalyzedExpr>,
    },
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
