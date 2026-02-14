//! Type system for ΓΛΩΣΣΑ
//!
//! Types in GLOSSA are derived from Greek nouns:
//! - ἀριθμός (arithmos) → Number (i64)
//! - ὄνομα (onoma) → String
//! - ἀληθές/ψεῦδος → Boolean
//! - λίστη (liste) → List/Vec
//!
//! Special types from Greek morphology:
//! - Optative mood → `Option<T>` (value that "might be")
//! - ἀποτέλεσμα (apotelasma) → `Result<T,E>` (outcome/result)

use super::Scope;
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

/// Map a genitive type name to a GlossaType
pub fn resolve_type_name(name: &str, scope: &Scope) -> GlossaType {
    match name {
        // ἀριθμοῦ (genitive of ἀριθμός) → Number
        "αριθμου" => GlossaType::Number,
        // ὀνόματος (genitive of ὄνομα) → String
        "ονοματος" => GlossaType::String,
        // λιστης (genitive of λίστα) → List
        "λιστης" => GlossaType::List(Box::new(GlossaType::Unknown)),
        _ => {
            // Check for user-defined types
            // Strip genitive ending and look up the nominative form
            // For now, just try the name as-is
            if let Some(ty) = scope.lookup_type(name) {
                ty.clone()
            } else {
                GlossaType::Unknown
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_to_greek() {
        assert_eq!(GlossaType::Number.to_greek(), "ἀριθμός");
        assert_eq!(GlossaType::String.to_greek(), "ὄνομα");
    }

    #[test]
    fn test_type_compatibility() {
        assert!(GlossaType::Number.is_compatible(&GlossaType::Number));
        assert!(!GlossaType::Number.is_compatible(&GlossaType::String));
        assert!(GlossaType::Unknown.is_compatible(&GlossaType::Number));
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
