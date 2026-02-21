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

use crate::morphology::{Case, Gender, Number, Person};
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
    #[error("Σφάλμα σημασίας: {message}")]
    #[diagnostic(code(glossa::semantic))]
    SemanticError { message: String },

    /// **Undefined Name**: You tried to use a variable or function that doesn't exist.
    ///
    /// Remember to define variables with `ἔστω` (let be) before using them.
    #[error("Ἄγνωστον ὄνομα: {name}")]
    #[diagnostic(code(glossa::undefined))]
    UndefinedName { name: String },

    /// **Agreement Error**: Grammatical agreement failed.
    ///
    /// In Greek, the Subject must agree with the Verb in Person and Number.
    /// Adjectives must agree with Nouns in Gender, Number, and Case.
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

/// Result type for ΓΛΩΣΣΑ operations
pub type GlossaResult<T> = Result<T, GlossaError>;

// --- Messages from src/errors/messages.rs ---

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
