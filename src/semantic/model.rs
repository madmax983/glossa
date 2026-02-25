//! Semantic AST and Data Models
//!
//! This module contains the data structures that represent the analyzed program.
//! It decouples the data from the logic to prevent circular dependencies.

use crate::morphology::Gender;
use smol_str::SmolStr;

/// Types in ΓΛΩΣΣΑ
///
/// This enum represents the type system of the language.
/// It maps directly to Rust types but uses Greek terminology.
///
/// # Type Compatibility
///
/// Types are checked for compatibility using [`GlossaType::is_compatible`].
/// `GlossaType::Unknown` acts as a wildcard that matches any type (useful during inference).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GlossaType {
    /// **ἀριθμός** (Number) - 64-bit signed integer (`i64`)
    ///
    /// # Examples
    /// `1`, `42`, `-5`, `μηδέν` (zero)
    Number,

    /// **ὄνομα** (Name/String) - UTF-8 string (`String`)
    ///
    /// # Examples
    /// `«χαῖρε»` ("hello")
    String,

    /// **ἀληθές/ψεῦδος** (Boolean) - `bool`
    ///
    /// # Examples
    /// `ἀληθές` (true), `ψεῦδος` (false)
    Boolean,

    /// **λίστη** (List) - Dynamic array (`Vec<T>`)
    ///
    /// # Examples
    /// `[1, 2, 3]` (`List<Number>`)
    List(Box<GlossaType>),

    /// **σύνολον** (Set) - Hash set (`HashSet<T>`)
    ///
    /// Stores unique elements.
    Set(Box<GlossaType>),

    /// **χάρτης** (Map) - Hash map (`HashMap<K, V>`)
    ///
    /// Key-value storage.
    Map(Box<GlossaType>, Box<GlossaType>),

    /// `Option<T>` - value that might not exist
    ///
    /// Expressed in ΓΛΩΣΣΑ via the **optative mood** (εὑρεθείη "might be found").
    /// The optative mood in Ancient Greek expresses wish, possibility, or potential,
    /// making it a natural fit for optional values.
    ///
    /// # Examples
    /// - `τί` (ti) → `Some(value)` ("something")
    /// - `οὐδέν` (ouden) → `None` ("nothing")
    /// - `;` after optative verb → propagates `None` (like Rust's `?`)
    Option(Box<GlossaType>),

    /// `Result<T, E>` - value or error
    ///
    /// Expressed in ΓΛΩΣΣΑ via **ἀποτέλεσμα** (apotelasma) "result/outcome".
    /// Uses disjunctive patterns to distinguish success from failure.
    ///
    /// # Examples
    /// - `ἐπιτυχία` (epitychia) → `Ok(value)` ("success")
    /// - `σφάλμα` (sphalma) → `Err(error)` ("error/mistake")
    /// - `;` after Result expression → propagates `Err` (like Rust's `?`)
    Result(Box<GlossaType>, Box<GlossaType>),

    /// **εἶδος** (Form/Type) - User-defined struct
    ///
    /// Named types defined by the user (e.g., `εἶδος Χρήστης`).
    Struct {
        name: SmolStr,
        gender: Gender,
        fields: Vec<(SmolStr, GlossaType)>,
    },

    /// **ἔργον** (Function) - Function signature
    ///
    /// Represents the type of a function, including parameter and return types.
    Function {
        params: Vec<GlossaType>,
        returns: Box<GlossaType>,
    },

    /// **οὐδέν** (Nothing) - Unit type `()`
    ///
    /// Represents the absence of a value (void).
    Unit,

    /// **ἄγνωστον** (Unknown) - Type inference placeholder
    ///
    /// Used when the type cannot yet be determined. Acts as a wildcard in compatibility checks.
    Unknown,
}

impl std::fmt::Display for GlossaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlossaType::Number => write!(f, "Ἀριθμός"),
            GlossaType::String => write!(f, "Ὄνομα"),
            GlossaType::Boolean => write!(f, "Ἀληθές/Ψεῦδος"),
            GlossaType::List(inner) => write!(f, "Λίστη<{}>", inner),
            GlossaType::Set(inner) => write!(f, "Σύνολον<{}>", inner),
            GlossaType::Map(k, v) => write!(f, "Χάρτης<{}, {}>", k, v),
            GlossaType::Option(inner) => write!(f, "Εὑρεθείη<{}>", inner),
            GlossaType::Result(ok, err) => write!(f, "Ἀποτέλεσμα<{}, {}>", ok, err),
            GlossaType::Struct { name, .. } => write!(f, "Εἶδος {}", name),
            GlossaType::Function { params, returns } => {
                let params_str: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                write!(f, "Ἔργον({}) -> {}", params_str.join(", "), returns)
            }
            GlossaType::Unit => write!(f, "Οὐδέν"),
            GlossaType::Unknown => write!(f, "Ἄγνωστον"),
        }
    }
}

impl GlossaType {
    /// Get the Greek name for this type
    pub fn to_greek(&self) -> &'static str {
        match self {
            GlossaType::Number => "ἀριθμός",
            GlossaType::String => "ὄνομα",
            GlossaType::Boolean => "ἀληθές",
            GlossaType::List(_) => "λίστη",
            GlossaType::Set(_) => "σύνολον",
            GlossaType::Map(_, _) => "χάρτης",
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
            (GlossaType::Set(a), GlossaType::Set(b)) => a.is_compatible(b),
            (GlossaType::Map(k1, v1), GlossaType::Map(k2, v2)) => {
                k1.is_compatible(k2) && v1.is_compatible(v2)
            }
            (GlossaType::Option(a), GlossaType::Option(b)) => a.is_compatible(b),
            (GlossaType::Result(ok1, err1), GlossaType::Result(ok2, err2)) => {
                ok1.is_compatible(ok2) && err1.is_compatible(err2)
            }
            _ => self == other,
        }
    }
}

/// Detect built-in collection types (HashSet, HashMap)
///
/// Returns a tuple of (Rust collection name, GlossaType).
///
/// # Examples
///
/// ```
/// use glossa::semantic::detect_collection_type;
/// use glossa::semantic::GlossaType;
///
/// let (name, ty) = detect_collection_type("χαρτης").unwrap();
/// assert_eq!(name, "HashMap");
/// assert!(matches!(ty, GlossaType::Map(..)));
/// ```
pub fn detect_collection_type(type_name: &str) -> Option<(&'static str, GlossaType)> {
    match type_name {
        "συνολον" => Some(("HashSet", GlossaType::Set(Box::new(GlossaType::Unknown)))),
        "χαρτης" => Some((
            "HashMap",
            GlossaType::Map(Box::new(GlossaType::Unknown), Box::new(GlossaType::Unknown)),
        )),
        _ => None,
    }
}

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

// --- Trait and Type Definitions ---

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_to_greek() {
        assert_eq!(GlossaType::Number.to_greek(), "ἀριθμός");
        assert_eq!(GlossaType::String.to_greek(), "ὄνομα");
        assert_eq!(GlossaType::Boolean.to_greek(), "ἀληθές");
        assert_eq!(
            GlossaType::List(Box::new(GlossaType::Number)).to_greek(),
            "λίστη"
        );
        assert_eq!(
            GlossaType::Set(Box::new(GlossaType::Number)).to_greek(),
            "σύνολον"
        );
        assert_eq!(
            GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number)).to_greek(),
            "χάρτης"
        );
        assert_eq!(
            GlossaType::Option(Box::new(GlossaType::Number)).to_greek(),
            "εὑρεθείη"
        );
        assert_eq!(
            GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String))
                .to_greek(),
            "ἀποτέλεσμα"
        );
        assert_eq!(GlossaType::Unit.to_greek(), "οὐδέν");
        assert_eq!(
            GlossaType::Struct {
                name: "User".into(),
                gender: Gender::Masculine,
                fields: vec![]
            }
            .to_greek(),
            "εἶδος"
        );
        assert_eq!(
            GlossaType::Function {
                params: vec![],
                returns: Box::new(GlossaType::Unit)
            }
            .to_greek(),
            "ἔργον"
        );
        assert_eq!(GlossaType::Unknown.to_greek(), "ἄγνωστον");
    }

    #[test]
    fn test_type_compatibility() {
        // Base types
        assert!(GlossaType::Number.is_compatible(&GlossaType::Number));
        assert!(!GlossaType::Number.is_compatible(&GlossaType::String));
        assert!(GlossaType::Boolean.is_compatible(&GlossaType::Boolean));

        // Unknown compatibility
        assert!(GlossaType::Unknown.is_compatible(&GlossaType::Number));
        assert!(GlossaType::String.is_compatible(&GlossaType::Unknown));

        // List compatibility
        let list_num = GlossaType::List(Box::new(GlossaType::Number));
        let list_str = GlossaType::List(Box::new(GlossaType::String));
        let list_unknown = GlossaType::List(Box::new(GlossaType::Unknown));

        assert!(list_num.is_compatible(&list_num));
        assert!(!list_num.is_compatible(&list_str));
        assert!(list_num.is_compatible(&list_unknown));
        assert!(list_unknown.is_compatible(&list_num));

        // Set compatibility
        let set_num = GlossaType::Set(Box::new(GlossaType::Number));
        let set_unknown = GlossaType::Set(Box::new(GlossaType::Unknown));
        assert!(set_num.is_compatible(&set_unknown));

        // Map compatibility
        let map_s_n =
            GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number));
        let map_s_u =
            GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Unknown));
        let map_u_n =
            GlossaType::Map(Box::new(GlossaType::Unknown), Box::new(GlossaType::Number));

        assert!(map_s_n.is_compatible(&map_s_u));
        assert!(map_s_n.is_compatible(&map_u_n));
        assert!(!map_s_n.is_compatible(&GlossaType::Map(
            Box::new(GlossaType::Number),
            Box::new(GlossaType::Number)
        )));

        // Option compatibility
        let opt_num = GlossaType::Option(Box::new(GlossaType::Number));
        let opt_unknown = GlossaType::Option(Box::new(GlossaType::Unknown));
        assert!(opt_num.is_compatible(&opt_unknown));

        // Result compatibility
        let res_n_s = GlossaType::Result(
            Box::new(GlossaType::Number),
            Box::new(GlossaType::String),
        );
        let res_u_s = GlossaType::Result(
            Box::new(GlossaType::Unknown),
            Box::new(GlossaType::String),
        );
        let res_n_u = GlossaType::Result(
            Box::new(GlossaType::Number),
            Box::new(GlossaType::Unknown),
        );

        assert!(res_n_s.is_compatible(&res_u_s));
        assert!(res_n_s.is_compatible(&res_n_u));
        assert!(!res_n_s.is_compatible(&GlossaType::Result(
            Box::new(GlossaType::Boolean),
            Box::new(GlossaType::String)
        )));
    }

    #[test]
    fn test_detect_collection_type() {
        assert!(detect_collection_type("συνολον").is_some());
        assert!(detect_collection_type("χαρτης").is_some());
        assert!(detect_collection_type("other").is_none());
    }

    #[test]
    fn test_display_formatting() {
        assert_eq!(format!("{}", GlossaType::Number), "Ἀριθμός");
        assert_eq!(format!("{}", GlossaType::String), "Ὄνομα");
        assert_eq!(format!("{}", GlossaType::Boolean), "Ἀληθές/Ψεῦδος");
        assert_eq!(
            format!("{}", GlossaType::List(Box::new(GlossaType::Number))),
            "Λίστη<Ἀριθμός>"
        );
        assert_eq!(
            format!("{}", GlossaType::Set(Box::new(GlossaType::Number))),
            "Σύνολον<Ἀριθμός>"
        );
        assert_eq!(
            format!(
                "{}",
                GlossaType::Map(
                    Box::new(GlossaType::String),
                    Box::new(GlossaType::Option(Box::new(GlossaType::Number)))
                )
            ),
            "Χάρτης<Ὄνομα, Εὑρεθείη<Ἀριθμός>>"
        );
        assert_eq!(
            format!(
                "{}",
                GlossaType::Result(Box::new(GlossaType::Unit), Box::new(GlossaType::String))
            ),
            "Ἀποτέλεσμα<Οὐδέν, Ὄνομα>"
        );
        assert_eq!(
            format!(
                "{}",
                GlossaType::Struct {
                    name: "User".into(),
                    gender: Gender::Masculine,
                    fields: vec![]
                }
            ),
            "Εἶδος User"
        );
        assert_eq!(
            format!(
                "{}",
                GlossaType::Function {
                    params: vec![GlossaType::Number, GlossaType::String],
                    returns: Box::new(GlossaType::Boolean)
                }
            ),
            "Ἔργον(Ἀριθμός, Ὄνομα) -> Ἀληθές/Ψεῦδος"
        );
        assert_eq!(format!("{}", GlossaType::Unit), "Οὐδέν");
        assert_eq!(format!("{}", GlossaType::Unknown), "Ἄγνωστον");
    }
}
