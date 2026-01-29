//! Type system for ΓΛΩΣΣΑ
//!
//! Types in GLOSSA are derived from Greek nouns:
//! - ἀριθμός (arithmos) → Number (i64)
//! - ὄνομα (onoma) → String
//! - ἀληθές/ψεῦδος → Boolean
//! - λίστη (liste) → List/Vec
//!
//! Special types from Greek morphology:
//! - Optative mood → Option<T> (value that "might be")
//! - ἀποτέλεσμα (apotelasma) → Result<T,E> (outcome/result)

use crate::morphology::{Case, Gender, Tense};

/// Types in ΓΛΩΣΣΑ
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GlossaType {
    /// ἀριθμός - number (i64)
    Number,

    /// ὄνομα - string
    String,

    /// ἀληθές/ψεῦδος - boolean
    Boolean,

    /// λίστη - list of T
    List(Box<GlossaType>),

    /// Option<T> - value that might not exist
    ///
    /// Expressed in ΓΛΩΣΣΑ via the **optative mood** (εὑρεθείη "might be found").
    /// The optative mood in Ancient Greek expresses wish, possibility, or potential,
    /// making it a natural fit for optional values.
    ///
    /// # Examples
    /// - τί (ti) → Some(value) ("something")
    /// - οὐδέν (ouden) → None ("nothing")
    /// - `;` after optative verb → propagates None (like Rust's `?`)
    /// - `!` suffix → unwrap (confident extraction)
    Option(Box<GlossaType>),

    /// Result<T, E> - value or error
    ///
    /// Expressed in ΓΛΩΣΣΑ via ἀποτέλεσμα (apotelasma) "result/outcome".
    /// Uses disjunctive patterns to distinguish success from failure.
    ///
    /// # Examples
    /// - ἐπιτυχία (epitychia) → Ok(value) ("success")
    /// - σφάλμα (sphalma) → Err(error) ("error/mistake")
    /// - `;` after Result expression → propagates Err (like Rust's `?`)
    Result(Box<GlossaType>, Box<GlossaType>),

    /// Custom struct type (from noun)
    Struct {
        name: std::string::String,
        gender: Gender,
        fields: Vec<(std::string::String, GlossaType)>,
    },

    /// Function type
    Function {
        params: Vec<GlossaType>,
        returns: Box<GlossaType>,
    },

    /// Unit type (void)
    Unit,

    /// Unknown/unresolved type
    Unknown,
}

impl GlossaType {
    /// Get the Rust equivalent type
    pub fn to_rust(&self) -> String {
        match self {
            GlossaType::Number => "i64".to_string(),
            GlossaType::String => "String".to_string(),
            GlossaType::Boolean => "bool".to_string(),
            GlossaType::List(inner) => format!("Vec<{}>", inner.to_rust()),
            GlossaType::Option(inner) => format!("Option<{}>", inner.to_rust()),
            GlossaType::Result(ok, err) => format!("Result<{}, {}>", ok.to_rust(), err.to_rust()),
            GlossaType::Unit => "()".to_string(),
            GlossaType::Struct { .. } => "struct".to_string(),
            GlossaType::Function { .. } => "fn".to_string(),
            GlossaType::Unknown => "_".to_string(),
        }
    }

    /// Get the Greek name for this type
    pub fn to_greek(&self) -> &'static str {
        match self {
            GlossaType::Number => "ἀριθμός",
            GlossaType::String => "ὄνομα",
            GlossaType::Boolean => "ἀληθές",
            GlossaType::List(_) => "λίστη",
            GlossaType::Option(_) => "εὑρεθείη", // "might be found" (optative)
            GlossaType::Result(_, _) => "ἀποτέλεσμα", // "result/outcome"
            GlossaType::Unit => "οὐδέν",
            GlossaType::Struct { .. } => "εἶδος",
            GlossaType::Function { .. } => "ἔργον",
            GlossaType::Unknown => "ἄγνωστον",
        }
    }

    /// Check if this type is compatible with another
    pub fn is_compatible(&self, other: &GlossaType) -> bool {
        match (self, other) {
            (GlossaType::Unknown, _) | (_, GlossaType::Unknown) => true,
            (GlossaType::List(a), GlossaType::List(b)) => a.is_compatible(b),
            (GlossaType::Option(a), GlossaType::Option(b)) => a.is_compatible(b),
            (GlossaType::Result(ok1, err1), GlossaType::Result(ok2, err2)) => {
                ok1.is_compatible(ok2) && err1.is_compatible(err2)
            }
            _ => self == other,
        }
    }
}

/// Ownership mode derived from grammatical case
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    /// Move semantics (accusative, aorist)
    Move,
    /// Immutable borrow (genitive)
    Borrow,
    /// Mutable borrow (dative)
    BorrowMut,
    /// Copy (for Copy types)
    Copy,
}

impl Ownership {
    /// Derive ownership from case
    pub fn from_case(case: Case) -> Self {
        match case {
            Case::Genitive => Ownership::Borrow,
            Case::Dative => Ownership::BorrowMut,
            Case::Accusative => Ownership::Move,
            _ => Ownership::Copy,
        }
    }

    /// Get the Rust reference prefix
    pub fn to_rust_prefix(&self) -> &'static str {
        match self {
            Ownership::Move => "",
            Ownership::Borrow => "&",
            Ownership::BorrowMut => "&mut ",
            Ownership::Copy => "",
        }
    }
}

/// Execution mode derived from verbal aspect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Present tense: streaming/iterative
    Streaming,
    /// Aorist tense: one-shot/complete
    OneShot,
    /// Perfect tense: cached/memoized
    Cached,
    /// Future tense: lazy/deferred
    Lazy,
}

impl ExecutionMode {
    pub fn from_tense(tense: Tense) -> Self {
        match tense {
            Tense::Present => ExecutionMode::Streaming,
            Tense::Aorist => ExecutionMode::OneShot,
            Tense::Perfect => ExecutionMode::Cached,
            Tense::Future => ExecutionMode::Lazy,
            _ => ExecutionMode::OneShot,
        }
    }
}

/// Infer type from a Greek word (by looking at lexicon or morphology)
pub fn infer_type(word: &str) -> GlossaType {
    let normalized = crate::grammar::normalize_greek(word);

    // Check lexicon first
    if let Some(entry) = crate::morphology::lexicon::lookup(&normalized) {
        match entry.rust_equiv {
            Some("i64") => return GlossaType::Number,
            Some("String") => return GlossaType::String,
            Some("bool") | Some("true") | Some("false") => return GlossaType::Boolean,
            Some("Vec") => return GlossaType::List(Box::new(GlossaType::Unknown)),
            _ => {}
        }
    }

    // Check for numeral
    if crate::morphology::lexicon::numeral_value(&normalized).is_some() {
        return GlossaType::Number;
    }

    GlossaType::Unknown
}

/// Trait definition for semantic analysis
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: std::string::String,
    pub required_methods: Vec<MethodSignature>,
    pub default_methods: Vec<DefaultMethod>,
}

/// Method signature in a trait
#[derive(Debug, Clone, PartialEq)]
pub struct MethodSignature {
    pub name: std::string::String,
    pub params: Vec<(std::string::String, GlossaType)>,
    pub return_type: Option<GlossaType>,
    pub has_default: bool,
}

/// Default method with implementation
#[derive(Debug, Clone)]
pub struct DefaultMethod {
    pub signature: MethodSignature,
    pub body: Vec<crate::semantic::AnalyzedStatement>,
}

/// Trait implementation for a type
#[derive(Debug, Clone)]
pub struct TraitImpl {
    pub trait_name: std::string::String,
    pub type_name: std::string::String,
    pub methods: Vec<ImplMethod>,
}

/// Method implementation in a trait impl
#[derive(Debug, Clone)]
pub struct ImplMethod {
    pub name: std::string::String,
    pub params: Vec<(std::string::String, GlossaType)>,
    pub body: Vec<AnalyzedStatement>,
}

/// Analyzed program with resolved names and types
#[derive(Debug, Clone)]
pub struct AnalyzedProgram {
    pub statements: Vec<AnalyzedStatement>,
    pub scope: crate::semantic::Scope,
}

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
        name: String,
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
        variable: String,
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
        name: String,
        params: Vec<(String, Option<GlossaType>)>,
        body: Vec<AnalyzedStatement>,
        return_type: Option<GlossaType>,
    },
    /// Type definition: εἶδος name ὁρίζειν { fields }
    TypeDefinition {
        name: String,
        fields: Vec<(String, GlossaType)>,
    },
    /// Trait definition: χαρακτήρ name ὁρίζειν { methods }
    TraitDefinition {
        name: String,
        methods: Vec<AnalyzedTraitMethod>,
    },
    /// Trait implementation: εἶδος Type τῷ Trait ἐμπίπτειν { methods }
    TraitImplementation {
        trait_name: String,
        type_name: String,
        methods: Vec<AnalyzedImplMethod>,
    },
}

/// An analyzed method in a trait definition
#[derive(Debug, Clone)]
pub struct AnalyzedTraitMethod {
    pub name: String,
    pub params: Vec<(String, GlossaType)>,
    pub is_default: bool,
    pub body: Option<Vec<AnalyzedStatement>>, // Some for default methods, None for required
}

/// An analyzed method in a trait implementation
#[derive(Debug, Clone)]
pub struct AnalyzedImplMethod {
    pub name: String,
    pub params: Vec<(String, GlossaType)>,
    pub body: Vec<AnalyzedStatement>,
}

/// Analyzed expression with type information
#[derive(Debug, Clone)]
pub struct AnalyzedExpr {
    pub expr: AnalyzedExprKind,
    pub glossa_type: GlossaType,
}

/// Iterator operation in analyzed form
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

/// Kind of analyzed expression
#[derive(Debug, Clone)]
pub enum AnalyzedExprKind {
    StringLiteral(String),
    NumberLiteral(i64),
    BooleanLiteral(bool),
    Variable(String),
    PropertyAccess {
        owner: Box<AnalyzedExpr>,
        property: String,
    },
    VerbCall {
        verb: String,
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
    /// Some(value) - Option<T> constructor
    Some(Box<AnalyzedExpr>),
    /// None - Option<T> empty value
    None,
    /// Ok(value) - Result<T,E> success constructor
    Ok(Box<AnalyzedExpr>),
    /// Err(error) - Result<T,E> error constructor
    Err(Box<AnalyzedExpr>),
    /// Unwrap operator (!) - confident extraction from Option/Result
    Unwrap(Box<AnalyzedExpr>),
    /// Try operator (`;` after Option/Result) - propagates None/Err upward
    Try(Box<AnalyzedExpr>),
    /// Index access array[index]
    IndexAccess {
        array: Box<AnalyzedExpr>,
        index: Box<AnalyzedExpr>,
    },
    /// Function call to user-defined function
    FunctionCall {
        func: String,
        args: Vec<AnalyzedExpr>,
    },
    /// Method call receiver.method(args)
    MethodCall {
        receiver: Box<AnalyzedExpr>,
        method: String,
        args: Vec<AnalyzedExpr>,
    },
    /// Trait method call receiverου method args (from trait impl)
    TraitMethodCall {
        receiver: Box<AnalyzedExpr>,
        trait_name: String,
        method_name: String,
        args: Vec<AnalyzedExpr>,
    },
    /// Struct instantiation Type { field: value, ... }
    StructInstantiation {
        type_name: String,
        fields: Vec<String>, // Field names from struct definition
        args: Vec<AnalyzedExpr>,
    },
    /// Lambda/closure |params| body
    Lambda {
        params: Vec<String>,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_to_rust() {
        assert_eq!(GlossaType::Number.to_rust(), "i64".to_string());
        assert_eq!(GlossaType::String.to_rust(), "String".to_string());
        assert_eq!(GlossaType::Boolean.to_rust(), "bool".to_string());
    }

    #[test]
    fn test_type_to_greek() {
        assert_eq!(GlossaType::Number.to_greek(), "ἀριθμός");
        assert_eq!(GlossaType::String.to_greek(), "ὄνομα");
    }

    #[test]
    fn test_ownership_from_case() {
        assert_eq!(Ownership::from_case(Case::Genitive), Ownership::Borrow);
        assert_eq!(Ownership::from_case(Case::Dative), Ownership::BorrowMut);
        assert_eq!(Ownership::from_case(Case::Accusative), Ownership::Move);
    }

    #[test]
    fn test_ownership_prefix() {
        assert_eq!(Ownership::Borrow.to_rust_prefix(), "&");
        assert_eq!(Ownership::BorrowMut.to_rust_prefix(), "&mut ");
        assert_eq!(Ownership::Move.to_rust_prefix(), "");
    }

    #[test]
    fn test_infer_type_numeral() {
        assert_eq!(infer_type("πέντε"), GlossaType::Number);
        assert_eq!(infer_type("δέκα"), GlossaType::Number);
    }

    #[test]
    fn test_type_compatibility() {
        assert!(GlossaType::Number.is_compatible(&GlossaType::Number));
        assert!(!GlossaType::Number.is_compatible(&GlossaType::String));
        assert!(GlossaType::Unknown.is_compatible(&GlossaType::Number));
    }
}
