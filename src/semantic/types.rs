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

use crate::morphology::{Case, Gender};
use smol_str::SmolStr;

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

    /// σύνολον - set of T (HashSet)
    Set(Box<GlossaType>),

    /// χάρτης - map from K to V (HashMap)
    Map(Box<GlossaType>, Box<GlossaType>),

    /// `Option<T>` - value that might not exist
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

    /// `Result<T, E>` - value or error
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
        name: SmolStr,
        gender: Gender,
        fields: Vec<(SmolStr, GlossaType)>,
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
            GlossaType::Set(inner) => format!("HashSet<{}>", inner.to_rust()),
            GlossaType::Map(key, value) => {
                format!("HashMap<{}, {}>", key.to_rust(), value.to_rust())
            }
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

/// Infer type from a Greek word (by looking at lexicon or morphology)
pub fn infer_type(word: &str) -> GlossaType {
    let normalized = crate::text::normalize_greek(word);

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

/// Detect built-in collection types (HashSet, HashMap)
pub fn detect_collection_type(type_name: &str) -> Option<GlossaType> {
    match type_name {
        "συνολον" => Some(GlossaType::Set(Box::new(GlossaType::Unknown))),
        "χαρτης" => Some(GlossaType::Map(
            Box::new(GlossaType::Unknown),
            Box::new(GlossaType::Unknown),
        )),
        _ => None,
    }
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

    #[test]
    fn test_detect_collection_type() {
        assert!(detect_collection_type("συνολον").is_some());
        assert!(detect_collection_type("χαρτης").is_some());
        assert!(detect_collection_type("other").is_none());
    }
}
