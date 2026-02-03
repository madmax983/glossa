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
        methods: Vec<AnalyzedTraitMethod>,
    },
    /// Trait implementation: εἶδος Type τῷ Trait ἐμπίπτειν { methods }
    TraitImplementation {
        trait_name: SmolStr,
        type_name: SmolStr,
        methods: Vec<AnalyzedImplMethod>,
    },
}

/// An analyzed method in a trait definition (AST node)
#[derive(Debug, Clone)]
pub struct AnalyzedTraitMethod {
    pub name: SmolStr,
    pub params: Vec<(SmolStr, GlossaType)>,
    pub is_default: bool,
    pub body: Option<Vec<AnalyzedStatement>>, // Some for default methods, None for required
    pub return_type: Option<GlossaType>,
}

/// An analyzed method in a trait implementation (AST node)
#[derive(Debug, Clone)]
pub struct AnalyzedImplMethod {
    pub name: SmolStr,
    pub params: Vec<(SmolStr, GlossaType)>,
    pub body: Vec<AnalyzedStatement>,
    pub return_type: Option<GlossaType>,
}

/// Iterator operation for AnalyzedExpr
#[derive(Debug, Clone)]
pub enum AnalyzedIteratorOp {
    /// .iter() - create iterator
    Iter,
    /// .map(closure) - transform elements
    Map(Box<AnalyzedExpr>),
    /// .filter(closure) - select elements
    Filter(Box<AnalyzedExpr>),
    /// .find(closure) - find first matching element
    Find(Box<AnalyzedExpr>),
    /// .fold(init, closure) - reduce to single value
    Fold {
        init: Box<AnalyzedExpr>,
        closure: Box<AnalyzedExpr>,
    },
    /// .any(closure) - test if any element matches
    Any(Box<AnalyzedExpr>),
    /// .all(closure) - test if all elements match
    All(Box<AnalyzedExpr>),
    /// .collect() - collect into collection
    Collect,
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
        capture_mode: crate::ast::CaptureMode,
    },
    /// Iterator chain collection.iter().map(...).filter(...)
    IteratorChain {
        collection: Box<AnalyzedExpr>,
        ops: Vec<AnalyzedIteratorOp>,
    },
    /// Literal value (used in iterator ops, different from specific literals above)
    Literal(i64),
    /// Collection constructor (HashSet::new(), HashMap::new())
    CollectionNew {
        collection_type: String,
    },
    /// Collection contains check (set.contains(&x), map.contains_key(&k))
    CollectionContains {
        collection: Box<AnalyzedExpr>,
        element: Box<AnalyzedExpr>,
        is_map: bool,
    },
}

// --- Trait and Type Definitions (moved from types.rs) ---

/// Trait definition for semantic analysis
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: SmolStr,
    pub methods: Vec<AnalyzedTraitMethod>,
}

/// Trait implementation for a type
#[derive(Debug, Clone)]
pub struct TraitImpl {
    pub trait_name: SmolStr,
    pub type_name: SmolStr,
}
