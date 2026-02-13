//! Greek error messages
//!
//! # Philosophy: Errors as Dialogue
//!
//! In ΓΛΩΣΣΑ, error messages are not just debug output; they are part of the immersion.
//! Instead of technical jargon like "Type Mismatch", the compiler speaks to you
//! in Ancient Greek:
//!
//! * "Ἐδόκει ἀριθμὸν εὑρεῖν..." (I expected to find a number...)
//! * "Οὐκ οἶδα τὸ ὄνομα..." (I do not know the name...)
//!
//! This module centralizes these strings to ensure consistency and grammatical correctness.
//! We strive to be helpful, polite, but firm—like a strict grammaticus teaching a pupil.

use crate::morphology::{Case, Gender, Number};

/// Get a Greek message for a type mismatch
///
/// Returns: "Ἐδόκει {expected} εὑρεῖν, ἀλλ' εὗρον {got}"
///
/// # Examples
///
/// ```
/// use glossa::errors::type_mismatch;
///
/// let msg = type_mismatch("ἀριθμόν", "ὄνομα");
/// assert_eq!(msg, "Ἐδόκει ἀριθμόν εὑρεῖν, ἀλλ' εὗρον ὄνομα");
/// ```
pub fn type_mismatch(expected: &str, got: &str) -> String {
    format!("Ἐδόκει {} εὑρεῖν, ἀλλ' εὗρον {}", expected, got)
}

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
    fn test_type_mismatch_message() {
        let msg = type_mismatch("ἀριθμός", "ὄνομα");
        assert!(msg.contains("Ἐδόκει"));
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
