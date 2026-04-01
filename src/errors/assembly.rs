use crate::morphology::{Number, Person};
use miette::Diagnostic;
use thiserror::Error;

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

    /// Missing verb in the statement
    ///
    /// # Example
    /// `ὁ ἄνθρωπος` (The man)
    #[error("Ῥῆμα οὐχ εὑρέθη! Πᾶσα πρᾶξις ῥῆμα ἀπαιτεῖ.")]
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

    /// Resource limit exceeded to prevent denial of service
    ///
    /// # Example
    /// Too many adjectives in a single sentence
    #[error("Ὑπέρβασις ὁρίου: {resource} > {max}. Μηδὲν ἄγαν!")]
    #[diagnostic(code(glossa::assembly::limit_exceeded))]
    LimitExceeded { resource: String, max: usize },
}
