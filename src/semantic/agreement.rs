//! Gender, number, and case agreement checking
//!
//! In Ancient Greek (and ΓΛΩΣΣΑ), adjectives must agree with nouns in
//! gender, number, and case. This agreement serves as a form of type checking.

use crate::morphology::{Case, Gender, Number, analyze};

/// Check agreement between two words (e.g., adjective and noun)
pub fn check_agreement(word1: &str, word2: &str) -> Result<(), AgreementError> {
    let analysis1 = analyze(word1);
    let analysis2 = analyze(word2);

    // Check gender agreement
    if let (Some(g1), Some(g2)) = (analysis1.gender, analysis2.gender)
        && g1 != g2
    {
        return Err(AgreementError::GenderMismatch {
            word1: word1.to_string(),
            gender1: g1,
            word2: word2.to_string(),
            gender2: g2,
        });
    }

    // Check number agreement
    if let (Some(n1), Some(n2)) = (analysis1.number, analysis2.number)
        && n1 != n2
    {
        return Err(AgreementError::NumberMismatch {
            word1: word1.to_string(),
            number1: n1,
            word2: word2.to_string(),
            number2: n2,
        });
    }

    // Check case agreement
    if let (Some(c1), Some(c2)) = (analysis1.case, analysis2.case)
        && c1 != c2
    {
        return Err(AgreementError::CaseMismatch {
            word1: word1.to_string(),
            case1: c1,
            word2: word2.to_string(),
            case2: c2,
        });
    }

    Ok(())
}

/// Errors that can occur during agreement checking
#[derive(Debug, Clone, thiserror::Error)]
pub enum AgreementError {
    #[error("Gender mismatch: {word1} ({gender1:?}) vs {word2} ({gender2:?})")]
    GenderMismatch {
        word1: String,
        gender1: Gender,
        word2: String,
        gender2: Gender,
    },

    #[error("Number mismatch: {word1} ({number1:?}) vs {word2} ({number2:?})")]
    NumberMismatch {
        word1: String,
        number1: Number,
        word2: String,
        number2: Number,
    },

    #[error("Case mismatch: {word1} ({case1:?}) vs {word2} ({case2:?})")]
    CaseMismatch {
        word1: String,
        case1: Case,
        word2: String,
        case2: Case,
    },
}

impl AgreementError {
    /// Convert to a Greek error message
    pub fn to_greek(&self) -> String {
        match self {
            AgreementError::GenderMismatch {
                word1,
                gender1,
                word2,
                gender2,
            } => {
                let g1 = gender_to_greek(*gender1);
                let g2 = gender_to_greek(*gender2);
                format!("Σφάλμα γένους: {} ({}) πρὸς {} ({})", word1, g1, word2, g2)
            }
            AgreementError::NumberMismatch {
                word1,
                number1,
                word2,
                number2,
            } => {
                let n1 = number_to_greek(*number1);
                let n2 = number_to_greek(*number2);
                format!("Σφάλμα ἀριθμοῦ: {} ({}) πρὸς {} ({})", word1, n1, word2, n2)
            }
            AgreementError::CaseMismatch {
                word1,
                case1,
                word2,
                case2,
            } => {
                let c1 = case_to_greek(*case1);
                let c2 = case_to_greek(*case2);
                format!("Σφάλμα πτώσεως: {} ({}) πρὸς {} ({})", word1, c1, word2, c2)
            }
        }
    }
}

fn gender_to_greek(gender: Gender) -> &'static str {
    match gender {
        Gender::Masculine => "ἀρσενικόν",
        Gender::Feminine => "θηλυκόν",
        Gender::Neuter => "οὐδέτερον",
    }
}

fn number_to_greek(number: Number) -> &'static str {
    match number {
        Number::Singular => "ἑνικός",
        Number::Plural => "πληθυντικός",
    }
}

fn case_to_greek(case: Case) -> &'static str {
    match case {
        Case::Nominative => "ὀνομαστική",
        Case::Genitive => "γενική",
        Case::Dative => "δοτική",
        Case::Accusative => "αἰτιατική",
        Case::Vocative => "κλητική",
    }
}

/// Check if a verb agrees with its subject in number and person
pub fn check_verb_subject_agreement(verb: &str, subject: &str) -> Result<(), AgreementError> {
    let verb_analysis = analyze(verb);
    let subject_analysis = analyze(subject);

    // Check number agreement between verb and subject
    if let (Some(verb_num), Some(subj_num)) = (verb_analysis.number, subject_analysis.number)
        && verb_num != subj_num
    {
        return Err(AgreementError::NumberMismatch {
            word1: verb.to_string(),
            number1: verb_num,
            word2: subject.to_string(),
            number2: subj_num,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agreement_same_forms() {
        // Both nominative singular masculine
        let result = check_agreement("μεγαλος", "χρηστος");
        assert!(result.is_ok());
    }

    #[test]
    fn test_agreement_gender_mismatch() {
        // Feminine adjective with masculine noun
        let result = check_agreement("μεγαλη", "χρηστος");
        assert!(matches!(result, Err(AgreementError::GenderMismatch { .. })));
    }

    #[test]
    fn test_agreement_number_mismatch() {
        // Plural vs singular
        let result = check_agreement("χρηστοι", "λογος");
        assert!(matches!(result, Err(AgreementError::NumberMismatch { .. })));
    }

    #[test]
    fn test_greek_error_message() {
        let err = AgreementError::GenderMismatch {
            word1: "μεγαλη".to_string(),
            gender1: Gender::Feminine,
            word2: "χρηστος".to_string(),
            gender2: Gender::Masculine,
        };

        let msg = err.to_greek();
        assert!(msg.contains("Σφάλμα γένους"));
        assert!(msg.contains("θηλυκόν"));
        assert!(msg.contains("ἀρσενικόν"));
    }

    #[test]
    fn test_verb_subject_agreement() {
        // Both singular
        let result = check_verb_subject_agreement("λεγω", "χρηστος");
        assert!(result.is_ok());
    }
}
