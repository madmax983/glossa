//! Type system for ΓΛΩΣΣΑ
//!
//! Types in GLOSSA are derived from Greek nouns:
//! - ἀριθμός (arithmos) → Number (i64)
//! - ὄνομα (onoma) → String
//! - ἀληθές/ψεῦδος → Boolean
//! - λίστη (liste) → List/Vec

use crate::morphology::{Gender, Case, Tense};

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
    pub fn to_rust(&self) -> &'static str {
        match self {
            GlossaType::Number => "i64",
            GlossaType::String => "String",
            GlossaType::Boolean => "bool",
            GlossaType::List(_) => "Vec",
            GlossaType::Unit => "()",
            GlossaType::Struct { .. } => "struct",
            GlossaType::Function { .. } => "fn",
            GlossaType::Unknown => "_",
        }
    }

    /// Get the Greek name for this type
    pub fn to_greek(&self) -> &'static str {
        match self {
            GlossaType::Number => "ἀριθμός",
            GlossaType::String => "ὄνομα",
            GlossaType::Boolean => "ἀληθές",
            GlossaType::List(_) => "λίστη",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_to_rust() {
        assert_eq!(GlossaType::Number.to_rust(), "i64");
        assert_eq!(GlossaType::String.to_rust(), "String");
        assert_eq!(GlossaType::Boolean.to_rust(), "bool");
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
