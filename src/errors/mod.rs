//! Error handling for ΓΛΩΣΣΑ
//!
//! This module implements the "Errors as Dialogue" philosophy. In ΓΛΩΣΣΑ, errors are not
//! just debug traces; they are the compiler speaking to you in Ancient Greek.
//!
//! # Philosophy: The Strict Grammaticus
//!
//! The compiler acts as a strict Ancient Greek teacher ("Grammaticus"). When you make a mistake,
//! it doesn't just say "Type Error". It says:
//!
//! > *«Τὸ «ἄνθρωπος» (ὀνομαστική) οὐ συμφωνεῖ τῷ «λέγουσι» (πληθυντικός)»*
//! > (The "man" (nominative) does not agree with "they say" (plural))
//!
//! This immersion helps users internalize the grammar of the language.
//!
//! # Error Categories
//!
//! Errors are categorized by the phase of compilation:
//!
//! 1. **Σύνταξις (Syntax)**: The words are not in a valid order (Parsing).
//! 2. **Σημασία (Semantics)**: The words make sense individually but not together (Analysis).
//! 3. **Συναρμογή (Assembly)**: The Subject, Verb, and Object do not agree (Agreement).
//! 4. **Ὄνομα (Name)**: A variable or function name is unknown.
//! 5. **Κῶδιξ (Codegen)**: An error occurred while generating Rust code.
//!
//! # Recovery Guide
//!
//! If you encounter an error, here is how to interpret it:
//!
//! * **Ἀσυμφωνία (Disagreement)**: Check your Case and Number.
//!   - Did you use a Plural Verb with a Singular Subject?
//!   - Did you use an Adjective that doesn't match the Gender of the Noun?
//!
//! * **Διπλοῦν ... (Double ...)**: You have too many words for one slot.
//!   - `Διπλοῦν ὑποκείμενον`: You have two Subjects (Nominative nouns).
//!   - `Διπλοῦν ῥῆμα`: You have two Verbs.
//!
//! * **Οὐκ οἶδα τὸ ὄνομα**: You are using a variable that hasn't been defined with `ἔστω`.

#![allow(unused_assignments)]

pub mod assembly;
mod messages;

pub use assembly::*;
pub use messages::*;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// Main error type for ΓΛΩΣΣΑ
///
/// This enum aggregates all possible errors from the compiler pipeline.
/// It implements [`miette::Diagnostic`] to provide pretty-printed error reports with
/// source code snippets and labels.
#[derive(Debug, Clone, Error, Diagnostic)]
pub enum GlossaError {
    /// **Syntax Error**: The parser failed to understand the code structure.
    ///
    /// This usually means a missing period `.`, unmatched braces `{}`, or invalid characters.
    ///
    /// # Example Output
    /// ```text
    /// Σφάλμα συντάξεως: expected statement
    /// ```
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

    /// **Semantic Error**: The code is syntactically valid but semantically meaningless.
    ///
    /// Examples include invalid type conversions, recursion limits, or logical paradoxes.
    ///
    /// # Example Output
    /// ```text
    /// Σφάλμα σημασίας: Τὸ «x» οὐχ ὡρίσθη
    /// ```
    #[error("Σφάλμα σημασίας: {message}")]
    #[diagnostic(code(glossa::semantic))]
    SemanticError { message: String },

    /// **Undefined Name**: You tried to use a variable or function that doesn't exist.
    ///
    /// Remember to define variables with `ἔστω` (let be) before using them.
    ///
    /// # Example Output
    /// ```text
    /// Ἄγνωστον ὄνομα: ξ
    /// ```
    #[error("Ἄγνωστον ὄνομα: {name}")]
    #[diagnostic(code(glossa::undefined))]
    UndefinedName { name: String },

    /// **Agreement Error**: Grammatical agreement failed.
    ///
    /// In Greek, the Subject must agree with the Verb in Person and Number.
    /// Adjectives must agree with Nouns in Gender, Number, and Case.
    ///
    /// # Example Output
    /// ```text
    /// Σφάλμα συμφωνίας: ὑποκείμενον (Singular) ἀλλὰ ῥῆμα (Plural)
    /// ```
    #[error("Σφάλμα συμφωνίας: {message}")]
    #[diagnostic(code(glossa::agreement))]
    AgreementError { message: String },

    /// **Codegen Error**: Failed to generate valid Rust code.
    ///
    /// This is an internal error indicating the transpiler produced invalid output.
    #[error("Σφάλμα κώδικος: {message}")]
    #[diagnostic(code(glossa::codegen))]
    CodegenError { message: String },

    /// **Limit Exceeded**: Resource exhaustion protection.
    ///
    /// To prevent Denial of Service (DoS) attacks or infinite loops during compilation,
    /// we limit the depth of recursion and number of elements.
    #[error("Ὑπέρβασις ὀρίου: {resource} ({max})")]
    #[diagnostic(code(glossa::limit_exceeded))]
    LimitExceeded { resource: String, max: usize },

    /// **Assembly Error**: The semantic assembler failed to build a valid sentence.
    ///
    /// This includes "Double Subject", "Missing Verb", and other sentence-structure errors.
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

    /// Create a parse error with source context
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

    /// Get the Greek category name for this error
    pub fn category_greek(&self) -> &'static str {
        match self {
            GlossaError::ParseError { .. } => "Σύνταξις",
            GlossaError::SemanticError { .. } => "Σημασία",
            GlossaError::UndefinedName { .. } => "Ὄνομα",
            GlossaError::AgreementError { .. } => "Συμφωνία",
            GlossaError::CodegenError { .. } => "Κῶδιξ",
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
