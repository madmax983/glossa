//! Error handling for ΓΛΩΣΣΑ
//!
//! Provides Greek error messages with miette integration.

#![allow(unused_assignments)]

use crate::morphology::{Case, Gender, Number, Person};
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

// ==================================================================================
// MAIN ERROR TYPES
// ==================================================================================

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

    #[error("Ἄγνωστον ὄνομα: {name}")]
    #[diagnostic(code(glossa::undefined))]
    UndefinedName { name: String },

    #[error("Σφάλμα συμφωνίας: {message}")]
    #[diagnostic(code(glossa::agreement))]
    AgreementError { message: String },

    #[error("Σφάλμα κώδικος: {message}")]
    #[diagnostic(code(glossa::codegen))]
    CodegenError { message: String },

    #[error("Ὑπέρβασις ὀρίου: {resource} ({max})")]
    #[diagnostic(code(glossa::limit_exceeded))]
    LimitExceeded { resource: String, max: usize },

    #[error(transparent)]
    #[diagnostic(transparent)]
    AssemblyError(#[from] AssemblyError),
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

    /// Get the Greek error category
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

// ==================================================================================
// ASSEMBLY ERRORS
// ==================================================================================

/// Errors that can occur during assembly
#[derive(Debug, Clone, Error, Diagnostic)]
pub enum AssemblyError {
    /// Two subjects found in the same statement (Nominative collision)
    ///
    /// # Example
    /// `ὁ ἄνθρωπος ὁ θεὸς λέγει` (The man the god says)
    #[error("Διπλοῦν ὑποκείμενον! Δύο βασιλεῖς οὐ δύνανται μιᾶς πόλεως ἄρχειν.")]
    #[diagnostic(code(glossa::assembly::double_subject))]
    DoubleSubject,

    /// Two objects found in the same statement (Accusative collision)
    ///
    /// # Example
    /// `τὸν λόγον τὴν πόλιν βλέπω` (I see the word the city)
    #[error("Διπλοῦν ἀντικείμενον! Ἓν μόνον κατηγορεῖς.")]
    #[diagnostic(code(glossa::assembly::double_object))]
    DoubleObject,

    /// Two indirect objects found in the same statement (Dative collision)
    ///
    /// # Example
    /// `τῷ ἀνθρώπῳ τῷ θεῷ δίδωμι` (I give to the man to the god)
    #[error("Διπλοῦν ἔμμεσον αντικείμενον! Ἓν μόνον παραλήπτην ἔχεις.")]
    #[diagnostic(code(glossa::assembly::double_indirect))]
    DoubleIndirect,

    /// Two verbs found in the same statement
    ///
    /// # Example
    /// `λέγει γράφει ὁ ἄνθρωπος` (The man says writes)
    #[error("Διπλοῦν ῥῆμα! Μία πρᾶξις ἑκάστοτε.")]
    #[diagnostic(code(glossa::assembly::double_verb))]
    DoubleVerb,

    /// No verb found in the statement
    ///
    /// Note: Pure expressions (like `5`) are allowed, but incomplete sentences trigger this.
    ///
    /// # Example
    /// `ὁ ἄνθρωπος τὸν λόγον` (The man the word ... [missing action])
    #[error("Ῥῆμα οὐχ εὑρέθη! Οὐδὲν ἐγένετο.")]
    #[diagnostic(code(glossa::assembly::missing_verb))]
    MissingVerb,

    /// Subject and Verb do not agree in number/person
    ///
    /// # Example
    /// `ὁ ἄνθρωπος (Singular) λέγουσιν (Plural)`
    #[error("Ἀσυμφωνία: ὑποκείμενον {subject:?} ἀλλὰ ῥῆμα {verb:?}")]
    #[diagnostic(code(glossa::assembly::subject_verb_disagreement))]
    SubjectVerbDisagreement {
        subject: (Option<Person>, Option<Number>),
        verb: (Option<Person>, Option<Number>),
    },

    /// Adjective and Noun do not agree in gender
    ///
    /// # Example
    /// `ὁ καλὸς (Masc) γυνή (Fem)`
    #[error("Ἀσυμφωνία γένους: {word1} ({gender1:?}) πρὸς {word2} ({gender2:?})")]
    #[diagnostic(code(glossa::assembly::gender_mismatch))]
    GenderMismatch {
        word1: String,
        gender1: Gender,
        word2: String,
        gender2: Gender,
    },

    /// Resource limit exceeded to prevent denial of service
    ///
    /// # Example
    /// Too many adjectives in a single sentence
    #[error("Ὑπέρβασις ὁρίου: {resource} > {max}. Μηδὲν ἄγαν!")]
    #[diagnostic(code(glossa::assembly::limit_exceeded))]
    LimitExceeded { resource: String, max: usize },
}

// ==================================================================================
// MESSAGES
// ==================================================================================

/// Get a Greek message for an undefined variable
///
/// Returns: "Οὐκ οἶδα τὸ ὄνομα «{name}»"
///
/// # Examples
///
/// ```
/// use glossa::errors::undefined_variable;
///
/// let msg = undefined_variable("ξ");
/// assert_eq!(msg, "Οὐκ οἶδα τὸ ὄνομα «ξ»");
/// ```
pub fn undefined_variable(name: &str) -> String {
    format!("Οὐκ οἶδα τὸ ὄνομα «{}»", name)
}

/// Get a Greek message for assignment to immutable variable
///
/// Returns: "Τὸ «{name}» ἀμετάβλητόν ἐστιν — χρῆσον μετά πρὸ τοῦ ὁρισμοῦ"
///
/// # Examples
///
/// ```
/// use glossa::errors::immutable_assignment;
///
/// let msg = immutable_assignment("π");
/// assert!(msg.contains("ἀμετάβλητόν ἐστιν"));
/// ```
pub fn immutable_assignment(name: &str) -> String {
    format!(
        "Τὸ «{}» ἀμετάβλητόν ἐστιν — χρῆσον μετά πρὸ τοῦ ὁρισμοῦ",
        name
    )
}

/// Get a Greek message for gender mismatch
///
/// Returns: "Τὸ «{word1}» ({gender1}) οὐ συμφωνεῖ τῷ «{word2}» ({gender2})"
///
/// # Examples
///
/// ```
/// use glossa::errors::gender_mismatch;
/// use glossa::morphology::Gender;
///
/// let msg = gender_mismatch("καλός", Gender::Masculine, "γυνή", Gender::Feminine);
/// assert!(msg.contains("οὐ συμφωνεῖ"));
/// ```
pub fn gender_mismatch(word1: &str, gender1: Gender, word2: &str, gender2: Gender) -> String {
    format!(
        "Τὸ «{}» ({}) οὐ συμφωνεῖ τῷ «{}» ({})",
        word1, gender1, word2, gender2
    )
}

/// Get a Greek message for number mismatch
///
/// Returns: "Τὸ «{word1}» ({num1}) οὐ συμφωνεῖ τῷ «{word2}» ({num2})"
///
/// # Examples
///
/// ```
/// use glossa::errors::number_mismatch;
/// use glossa::morphology::Number;
///
/// let msg = number_mismatch("ἄνθρωπος", Number::Singular, "λέγουσι", Number::Plural);
/// assert!(msg.contains("οὐ συμφωνεῖ"));
/// ```
pub fn number_mismatch(word1: &str, num1: Number, word2: &str, num2: Number) -> String {
    format!(
        "Τὸ «{}» ({}) οὐ συμφωνεῖ τῷ «{}» ({})",
        word1, num1, word2, num2
    )
}

/// Get a Greek message for case mismatch
///
/// Returns: "Τὸ «{word1}» ({case1}) οὐ συμφωνεῖ τῷ «{word2}» ({case2})"
///
/// # Examples
///
/// ```
/// use glossa::errors::case_mismatch;
/// use glossa::morphology::Case;
///
/// let msg = case_mismatch("ἄνθρωπος", Case::Nominative, "λόγον", Case::Accusative);
/// assert!(msg.contains("οὐ συμφωνεῖ"));
/// ```
pub fn case_mismatch(word1: &str, case1: Case, word2: &str, case2: Case) -> String {
    format!(
        "Τὸ «{}» ({}) οὐ συμφωνεῖ τῷ «{}» ({})",
        word1, case1, word2, case2
    )
}

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

        let err = GlossaError::parse("test");
        assert_eq!(err.category_greek(), "Σύνταξις");

        let err = GlossaError::undefined("test");
        assert_eq!(err.category_greek(), "Ὄνομα");

        let err = GlossaError::agreement("test");
        assert_eq!(err.category_greek(), "Συμφωνία");

        let err = GlossaError::codegen("test");
        assert_eq!(err.category_greek(), "Κῶδιξ");

        let err = GlossaError::LimitExceeded {
            resource: "test".into(),
            max: 10,
        };
        assert_eq!(err.category_greek(), "Όριον");

        let err = GlossaError::AssemblyError(AssemblyError::DoubleSubject);
        assert_eq!(err.category_greek(), "Συναρμογή");
    }

    #[test]
    fn test_undefined_variable_message() {
        let msg = undefined_variable("ξ");
        assert!(msg.contains("Οὐκ οἶδα"));
        assert!(msg.contains("ξ"));
    }

    #[test]
    fn test_gender_mismatch_message() {
        let msg = gender_mismatch("μεγάλη", Gender::Feminine, "χρήστος", Gender::Masculine);
        assert!(msg.contains("οὐ συμφωνεῖ"));
    }
}
