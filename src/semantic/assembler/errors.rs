//! Errors for the semantic assembler
//!
//! This module defines errors that can occur during sentence assembly,
//! such as grammatical disagreement or duplicate constituents.

use crate::morphology::{Gender, Number, Person};
use thiserror::Error;

/// Errors that can occur during assembly
#[derive(Debug, Clone, Error)]
pub enum AssemblyError {
    /// Two subjects found in the same statement (Nominative collision)
    ///
    /// # Example
    /// `ὁ ἄνθρωπος ὁ θεὸς λέγει` (The man the god says)
    #[error("Διπλοῦν ὑποκείμενον! Δύο βασιλεῖς οὐ δύνανται μιᾶς πόλεως ἄρχειν.")]
    DoubleSubject,

    /// Two objects found in the same statement (Accusative collision)
    ///
    /// # Example
    /// `τὸν λόγον τὴν πόλιν βλέπω` (I see the word the city)
    #[error("Διπλοῦν ἀντικείμενον! Ἓν μόνον κατηγορεῖς.")]
    DoubleObject,

    /// Two verbs found in the same statement
    ///
    /// # Example
    /// `λέγει γράφει ὁ ἄνθρωπος` (The man says writes)
    #[error("Διπλοῦν ῥῆμα! Μία πρᾶξις ἑκάστοτε.")]
    DoubleVerb,

    /// No verb found in the statement
    ///
    /// Note: Pure expressions (like `5`) are allowed, but incomplete sentences trigger this.
    ///
    /// # Example
    /// `ὁ ἄνθρωπος τὸν λόγον` (The man the word ... [missing action])
    #[error("Ῥῆμα οὐχ εὑρέθη! Οὐδὲν ἐγένετο.")]
    MissingVerb,

    /// Subject and Verb do not agree in number/person
    ///
    /// # Example
    /// `ὁ ἄνθρωπος (Singular) λέγουσιν (Plural)`
    #[error("Ἀσυμφωνία: ὑποκείμενον {subject:?} ἀλλὰ ῥῆμα {verb:?}")]
    SubjectVerbDisagreement {
        subject: (Option<Person>, Option<Number>),
        verb: (Option<Person>, Option<Number>),
    },

    /// Adjective and Noun do not agree in gender
    ///
    /// # Example
    /// `ὁ καλὸς (Masc) γυνή (Fem)`
    #[error("Ἀσυμφωνία γένους: {word1} ({gender1:?}) πρὸς {word2} ({gender2:?})")]
    GenderMismatch {
        word1: String,
        gender1: Gender,
        word2: String,
        gender2: Gender,
    },
}
