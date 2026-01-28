//! Greek error messages
//!
//! Provides helpful error messages in Ancient Greek style.

use crate::morphology::{Case, Gender, Number};

/// Get a Greek message for a type mismatch
pub fn type_mismatch(expected: &str, got: &str) -> String {
    format!("Ἐδόκει {} εὑρεῖν, ἀλλ' εὗρον {}", expected, got)
}

/// Get a Greek message for an undefined variable
pub fn undefined_variable(name: &str) -> String {
    format!("Οὐκ οἶδα τὸ ὄνομα «{}»", name)
}

/// Get a Greek message for gender mismatch
pub fn gender_mismatch(word1: &str, gender1: Gender, word2: &str, gender2: Gender) -> String {
    format!(
        "Τὸ «{}» ({}) οὐ συμφωνεῖ τῷ «{}» ({})",
        word1,
        gender_name(gender1),
        word2,
        gender_name(gender2)
    )
}

/// Get a Greek message for number mismatch
pub fn number_mismatch(word1: &str, num1: Number, word2: &str, num2: Number) -> String {
    format!(
        "Τὸ «{}» ({}) οὐ συμφωνεῖ τῷ «{}» ({})",
        word1,
        number_name(num1),
        word2,
        number_name(num2)
    )
}

/// Get a Greek message for case mismatch
pub fn case_mismatch(word1: &str, case1: Case, word2: &str, case2: Case) -> String {
    format!(
        "Τὸ «{}» ({}) οὐ συμφωνεῖ τῷ «{}» ({})",
        word1,
        case_name(case1),
        word2,
        case_name(case2)
    )
}

/// Get the Greek name for a gender
pub fn gender_name(gender: Gender) -> &'static str {
    match gender {
        Gender::Masculine => "ἀρσενικόν",
        Gender::Feminine => "θηλυκόν",
        Gender::Neuter => "οὐδέτερον",
    }
}

/// Get the Greek name for a number
pub fn number_name(number: Number) -> &'static str {
    match number {
        Number::Singular => "ἑνικός",
        Number::Plural => "πληθυντικός",
    }
}

/// Get the Greek name for a case
pub fn case_name(case: Case) -> &'static str {
    match case {
        Case::Nominative => "ὀνομαστική",
        Case::Genitive => "γενική",
        Case::Dative => "δοτική",
        Case::Accusative => "αἰτιατική",
        Case::Vocative => "κλητική",
    }
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
