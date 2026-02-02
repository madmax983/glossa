//! Error handling for ΓΛΩΣΣΑ
//!
//! Provides Greek error messages with miette integration.

#![allow(unused_assignments)]

pub mod assembly;
mod messages;

pub use assembly::*;
pub use messages::*;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// Main error type for ΓΛΩΣΣΑ
#[derive(Debug, Clone, Error, Diagnostic)]
pub enum GlossaError {
    #[error("Σφάλμα συντάξεως: {message}")]
    #[diagnostic(code(glossa::parse))]
    ParseError {
        message: String,
        #[source_code]
        #[allow(dead_code)]
        src: String,
        #[label("ἐνταῦθα")]
        #[allow(dead_code)]
        span: Option<SourceSpan>,
    },

    #[error("Σφάλμα σημασίας: {message}")]
    #[diagnostic(code(glossa::semantic))]
    SemanticError { message: String },

    #[error("Σφάλμα τύπου: {message}")]
    #[diagnostic(code(glossa::type_error))]
    TypeError { message: String },

    #[error("Ἄγνωστον ὄνομα: {name}")]
    #[diagnostic(code(glossa::undefined))]
    UndefinedName { name: String },

    #[error("Σφάλμα συμφωνίας: {message}")]
    #[diagnostic(code(glossa::agreement))]
    AgreementError { message: String },

    #[error("Σφάλμα κώδικος: {message}")]
    #[diagnostic(code(glossa::codegen))]
    CodegenError { message: String },

    #[error("Σφάλμα ἀρχείου: {message}")]
    #[diagnostic(code(glossa::io))]
    IoError { message: String },

    #[error(transparent)]
    #[diagnostic(transparent)]
    AssemblyError(#[from] assembly::AssemblyError),
}

impl GlossaError {
    /// Create a parse error
    pub fn parse(message: impl Into<String>) -> Self {
        GlossaError::ParseError {
            message: message.into(),
            src: String::new(),
            span: None,
        }
    }

    /// Create a parse error with source
    pub fn parse_with_source(
        message: impl Into<String>,
        src: impl Into<String>,
        span: SourceSpan,
    ) -> Self {
        GlossaError::ParseError {
            message: message.into(),
            src: src.into(),
            span: Some(span),
        }
    }

    /// Create a semantic error
    pub fn semantic(message: impl Into<String>) -> Self {
        GlossaError::SemanticError {
            message: message.into(),
        }
    }

    /// Create a type error
    pub fn type_error(message: impl Into<String>) -> Self {
        GlossaError::TypeError {
            message: message.into(),
        }
    }

    /// Create an undefined name error
    pub fn undefined(name: impl Into<String>) -> Self {
        GlossaError::UndefinedName { name: name.into() }
    }

    /// Create an agreement error
    pub fn agreement(message: impl Into<String>) -> Self {
        GlossaError::AgreementError {
            message: message.into(),
        }
    }

    /// Create a codegen error
    pub fn codegen(message: impl Into<String>) -> Self {
        GlossaError::CodegenError {
            message: message.into(),
        }
    }

    /// Create an IO error
    pub fn io(message: impl Into<String>) -> Self {
        GlossaError::IoError {
            message: message.into(),
        }
    }

    /// Get the Greek error category
    pub fn category_greek(&self) -> &'static str {
        match self {
            GlossaError::ParseError { .. } => "Σύνταξις",
            GlossaError::SemanticError { .. } => "Σημασία",
            GlossaError::TypeError { .. } => "Τύπος",
            GlossaError::UndefinedName { .. } => "Ὄνομα",
            GlossaError::AgreementError { .. } => "Συμφωνία",
            GlossaError::CodegenError { .. } => "Κῶδιξ",
            GlossaError::IoError { .. } => "Ἀρχεῖον",
            GlossaError::AssemblyError(_) => "Συναρμογή",
        }
    }
}

/// Result type for ΓΛΩΣΣΑ operations
pub type GlossaResult<T> = Result<T, GlossaError>;

// Conversion from Parser errors
impl From<crate::parser::ParseError> for GlossaError {
    fn from(err: crate::parser::ParseError) -> Self {
        GlossaError::parse(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error() {
        let err = GlossaError::parse("unexpected token");
        assert!(err.to_string().contains("Σφάλμα συντάξεως"));
    }

    #[test]
    fn test_undefined_error() {
        let err = GlossaError::undefined("ξ");
        assert!(err.to_string().contains("Ἄγνωστον ὄνομα"));
        assert!(err.to_string().contains("ξ"));
    }

    #[test]
    fn test_category_greek() {
        let err = GlossaError::semantic("test");
        assert_eq!(err.category_greek(), "Σημασία");
    }
}
