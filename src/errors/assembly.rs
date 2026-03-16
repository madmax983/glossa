use crate::morphology::{Gender, Number, Person};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assembly_error_display() {
        let err1 = AssemblyError::DoubleSubject;
        assert!(err1.to_string().contains("Διπλοῦν ὑποκείμενον"));

        let err2 = AssemblyError::DoubleObject;
        assert!(err2.to_string().contains("Διπλοῦν ἀντικείμενον"));

        let err3 = AssemblyError::DoubleIndirect;
        assert!(err3.to_string().contains("Διπλοῦν ἔμμεσον αντικείμενον"));

        let err4 = AssemblyError::DoubleVerb;
        assert!(err4.to_string().contains("Διπλοῦν ῥῆμα"));

        let err5 = AssemblyError::MissingVerb;
        assert!(err5.to_string().contains("Ῥῆμα οὐχ εὑρέθη"));

        let err6 = AssemblyError::SubjectVerbDisagreement {
            subject: (Some(Person::Third), Some(Number::Singular)),
            verb: (Some(Person::Third), Some(Number::Plural)),
        };
        assert!(err6.to_string().contains("Ἀσυμφωνία"));

        let err7 = AssemblyError::GenderMismatch {
            word1: "word1".into(),
            gender1: Gender::Masculine,
            word2: "word2".into(),
            gender2: Gender::Feminine,
        };
        assert!(err7.to_string().contains("Ἀσυμφωνία γένους"));

        let err8 = AssemblyError::LimitExceeded {
            resource: "foo".into(),
            max: 42,
        };
        assert!(err8.to_string().contains("foo"));
    }
}
