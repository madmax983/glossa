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
    AssemblyError(#[from] AssemblyError),
}

impl GlossaError {
    /// Creates a new `GlossaError::ParseError`.
    ///
    /// This is used when the source code contains invalid syntax or characters,
    /// meaning the lexer/parser cannot convert it into a valid abstract syntax tree.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::errors::GlossaError;
    ///
    /// let error = GlossaError::parse("expected closing brace");
    /// assert!(matches!(error, GlossaError::ParseError { .. }));
    /// ```
    pub fn parse(message: impl Into<String>) -> Self {
        GlossaError::ParseError {
            message: message.into(),
            src: String::new(),
            span: None,
        }
    }

    /// Creates a new `GlossaError::ParseError` complete with source file context.
    ///
    /// By providing the source string and the span representing the exact location
    /// of the error, the compiler can print helpful error diagnostics with lines,
    /// carets, and context using the [`miette`] crate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::errors::GlossaError;
    /// use miette::SourceSpan;
    ///
    /// let source = "ἔστω x 5".to_string(); // Invalid: variables start with greek letters
    /// let error = GlossaError::parse_with_source(
    ///     "invalid variable name",
    ///     source.clone(),
    ///     SourceSpan::new(5.into(), 1_usize)
    /// );
    /// ```
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

    /// Creates a new `GlossaError::SemanticError`.
    ///
    /// This is used when code is syntactically valid but breaks logical language rules,
    /// such as type mismatches, calling non-functions, or missing required fields.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::errors::GlossaError;
    ///
    /// let error = GlossaError::semantic("type mismatch: expected Number, found String");
    /// assert!(matches!(error, GlossaError::SemanticError { .. }));
    /// ```
    pub fn semantic(message: impl Into<String>) -> Self {
        GlossaError::SemanticError {
            message: message.into(),
        }
    }

    /// Creates a new `GlossaError::UndefinedName`.
    ///
    /// This specific semantic error is thrown when code attempts to reference a variable,
    /// type, or function that hasn't been defined in the current or parent [`Scope`].
    ///
    /// [`Scope`]: crate::semantic::Scope
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::errors::GlossaError;
    ///
    /// let error = GlossaError::undefined("ἀριθμός");
    /// assert!(matches!(error, GlossaError::UndefinedName { .. }));
    /// ```
    pub fn undefined(name: impl Into<String>) -> Self {
        GlossaError::UndefinedName { name: name.into() }
    }

    /// Creates a new `GlossaError::AgreementError`.
    ///
    /// This error occurs when the morphological traits of linked words do not match.
    /// In Ancient Greek grammar, adjectives must agree with their nouns in gender,
    /// number, and case; subjects must agree with verbs in person and number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::errors::GlossaError;
    ///
    /// let error = GlossaError::agreement("Subject is singular but verb is plural");
    /// assert!(matches!(error, GlossaError::AgreementError { .. }));
    /// ```
    pub fn agreement(message: impl Into<String>) -> Self {
        GlossaError::AgreementError {
            message: message.into(),
        }
    }

    /// Creates a new `GlossaError::CodegenError`.
    ///
    /// This error represents an internal failure during the translation from the
    /// semantically validated abstract syntax tree into Rust source code.
    /// This generally indicates a compiler bug where a semantic construct
    /// lacks a corresponding code generation implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::errors::GlossaError;
    ///
    /// let error = GlossaError::codegen("Unsupported expression type during generation");
    /// assert!(matches!(error, GlossaError::CodegenError { .. }));
    /// ```
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

pub(crate) mod assembly;
pub use assembly::AssemblyError;

/// Help messages in Greek
pub mod help {
    /// Help for the binding construct
    pub const BINDING: &str = "Χρῆσις: ὄνομα τιμή ἔστω.
Παράδειγμα: ξ πέντε ἔστω.";

    /// Help for the print construct
    pub const PRINT: &str = "Χρῆσις: τιμή λέγε.
Παράδειγμα: «χαῖρε κόσμε» λέγε.";

    /// Help for cases
    pub const CASES: &str = "Πτώσεις καὶ σημασίαι:
• Ὀνομαστική - τὸ ὑποκείμενον
• Γενική - κτῆσις, δάνεισμα (&)
• Δοτική - δάνεισμα μεταβλητόν (&mut)
• Αἰτιατική - τὸ ἀντικείμενον, κίνησις";
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
