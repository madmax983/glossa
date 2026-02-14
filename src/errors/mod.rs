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
///
/// This enum captures all possible failures in the compiler pipeline, providing
/// Greek-localized error messages and diagnostics.
#[derive(Debug, Clone, Error, Diagnostic)]
pub enum GlossaError {
    /// Syntax error (Parsing phase)
    ///
    /// Occurs when the code violates the grammar rules.
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

    /// Semantic error (Analysis phase)
    ///
    /// General semantic issues like invalid control flow or logical contradictions.
    #[error("Σφάλμα σημασίας: {message}")]
    #[diagnostic(code(glossa::semantic))]
    SemanticError { message: String },

    /// Type error (Analysis phase)
    ///
    /// Mismatched types (e.g., adding a string to a number).
    #[error("Σφάλμα τύπου: {message}")]
    #[diagnostic(code(glossa::type_error))]
    TypeError { message: String },

    /// Undefined name error (Resolution phase)
    ///
    /// Occurs when referencing a variable or function that hasn't been defined.
    #[error("Ἄγνωστον ὄνομα: {name}")]
    #[diagnostic(code(glossa::undefined))]
    UndefinedName { name: String },

    /// Agreement error (Assembly phase)
    ///
    /// Grammatical agreement failure (e.g., Subject-Verb number mismatch).
    #[error("Σφάλμα συμφωνίας: {message}")]
    #[diagnostic(code(glossa::agreement))]
    AgreementError { message: String },

    /// Code generation error (Codegen phase)
    ///
    /// Issues translating the analyzed program to Rust (e.g., invalid identifier).
    #[error("Σφάλμα κώδικος: {message}")]
    #[diagnostic(code(glossa::codegen))]
    CodegenError { message: String },

    /// I/O error (System)
    ///
    /// File reading/writing failures.
    #[error("Σφάλμα ἀρχείου: {message}")]
    #[diagnostic(code(glossa::io))]
    IoError { message: String },

    /// Resource limit exceeded (DoS Protection)
    ///
    /// Triggered when the program exceeds safety limits (recursion depth, file size).
    #[error("Ὑπέρβασις ὀρίου: {resource} ({max})")]
    #[diagnostic(code(glossa::limit_exceeded))]
    LimitExceeded { resource: String, max: usize },

    /// Assembly error (Assembly phase)
    ///
    /// Wraps errors from the [`crate::semantic::Assembler`].
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
            GlossaError::LimitExceeded { .. } => "Όριον",
            GlossaError::AssemblyError(_) => "Συναρμογή",
        }
    }
}

/// Result type for ΓΛΩΣΣΑ operations
pub type GlossaResult<T> = Result<T, GlossaError>;

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
