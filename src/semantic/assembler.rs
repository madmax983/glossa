//! Slot-based sentence assembler for ΓΛΩΣΣΑ
//!
//! This module implements a Greek-native approach to sentence parsing.
//! Instead of relying on word order, it routes words to slots based on
//! their grammatical case - just like Ancient Greek actually works.
//!
//! The assembler accumulates morphologically-analyzed tokens and assembles
//! them into a statement when finalized (at end of sentence).

use crate::morphology::{MorphAnalysis, PartOfSpeech, Case, Number, Gender, Person, Tense, Mood};
use crate::grammar::normalize_greek;

/// A fully assembled statement with all grammatical roles filled
#[derive(Debug, Clone)]
pub struct AssembledStatement {
    /// The subject (nominative) - the agent/doer
    pub subject: Option<Constituent>,
    /// The verb - the action
    pub verb: Option<VerbConstituent>,
    /// The direct object (accusative) - receives the action
    pub object: Option<Constituent>,
    /// The indirect object (dative) - recipient/beneficiary
    pub indirect: Option<Constituent>,
    /// Possessors/sources (genitive) - attached to other constituents
    pub genitives: Vec<Constituent>,
    /// Literal values (strings, numbers) that appeared
    pub literals: Vec<Literal>,
    /// Whether this is a query (ends with ;)
    pub is_query: bool,
}

/// A noun/pronoun constituent with its grammatical info
#[derive(Debug, Clone)]
pub struct Constituent {
    /// The dictionary form
    pub lemma: String,
    /// Original text as it appeared
    pub original: String,
    /// Grammatical case
    pub case: Case,
    /// Grammatical number
    pub number: Option<Number>,
    /// Grammatical gender
    pub gender: Option<Gender>,
}

/// A verb constituent with its grammatical info
#[derive(Debug, Clone)]
pub struct VerbConstituent {
    /// The dictionary form (1st person singular present)
    pub lemma: String,
    /// Original text as it appeared
    pub original: String,
    /// Person (1st, 2nd, 3rd)
    pub person: Option<Person>,
    /// Number (singular, plural)
    pub number: Option<Number>,
    /// Tense (present, aorist, etc.)
    pub tense: Option<Tense>,
    /// Mood (indicative, imperative, etc.)
    pub mood: Option<Mood>,
}

/// A literal value
#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(i64),
    Boolean(bool),
}

/// The slot-based assembler
///
/// Feed it tokens one by one, and it routes them to the appropriate slot
/// based on their grammatical case. When you hit end-of-statement, call
/// `finalize()` to get the assembled statement.
pub struct Assembler {
    pending_subject: Option<Constituent>,
    pending_object: Option<Constituent>,
    pending_indirect: Option<Constituent>,
    pending_verb: Option<VerbConstituent>,
    pending_genitives: Vec<Constituent>,
    pending_literals: Vec<Literal>,
    is_query: bool,
}

/// Errors that can occur during assembly
#[derive(Debug, Clone, thiserror::Error)]
pub enum AssemblyError {
    #[error("Διπλοῦν ὑποκείμενον! Δύο βασιλεῖς οὐ δύνανται μιᾶς πόλεως ἄρχειν.")]
    DoubleSubject,

    #[error("Διπλοῦν ἀντικείμενον! Ἓν μόνον κατηγορεῖς.")]
    DoubleObject,

    #[error("Διπλοῦν ῥῆμα! Μία πρᾶξις ἑκάστοτε.")]
    DoubleVerb,

    #[error("Ῥῆμα οὐχ εὑρέθη! Οὐδὲν ἐγένετο.")]
    MissingVerb,

    #[error("Ἀσυμφωνία: ὑποκείμενον {subject:?} ἀλλὰ ῥῆμα {verb:?}")]
    SubjectVerbDisagreement {
        subject: (Option<Person>, Option<Number>),
        verb: (Option<Person>, Option<Number>),
    },

    #[error("Ἀσυμφωνία γένους: {word1} ({gender1:?}) πρὸς {word2} ({gender2:?})")]
    GenderMismatch {
        word1: String,
        gender1: Gender,
        word2: String,
        gender2: Gender,
    },
}

impl Assembler {
    /// Create a new empty assembler
    pub fn new() -> Self {
        Assembler {
            pending_subject: None,
            pending_object: None,
            pending_indirect: None,
            pending_verb: None,
            pending_genitives: Vec::new(),
            pending_literals: Vec::new(),
            is_query: false,
        }
    }

    /// Reset the assembler for a new statement
    pub fn reset(&mut self) {
        self.pending_subject = None;
        self.pending_object = None;
        self.pending_indirect = None;
        self.pending_verb = None;
        self.pending_genitives.clear();
        self.pending_literals.clear();
        self.is_query = false;
    }

    /// Mark this statement as a query
    pub fn set_query(&mut self, is_query: bool) {
        self.is_query = is_query;
    }

    /// Feed a morphologically-analyzed token into the assembler
    pub fn feed(&mut self, analysis: &MorphAnalysis, original: &str) -> Result<(), AssemblyError> {
        match analysis.part_of_speech {
            PartOfSpeech::Noun | PartOfSpeech::Pronoun | PartOfSpeech::Adjective => {
                self.handle_nominal(analysis, original)
            }
            PartOfSpeech::Verb => {
                self.handle_verb(analysis, original)
            }
            PartOfSpeech::Numeral => {
                // Numerals can act as nouns or adjectives
                if let Some(value) = crate::morphology::lexicon::numeral_value(&normalize_greek(original)) {
                    self.pending_literals.push(Literal::Number(value));
                } else {
                    self.handle_nominal(analysis, original)?;
                }
                Ok(())
            }
            _ => Ok(()), // Ignore particles, articles for now
        }
    }

    /// Feed a string literal
    pub fn feed_string(&mut self, value: String) {
        self.pending_literals.push(Literal::String(value));
    }

    /// Feed a number literal
    pub fn feed_number(&mut self, value: i64) {
        self.pending_literals.push(Literal::Number(value));
    }

    /// Feed a boolean literal
    pub fn feed_boolean(&mut self, value: bool) {
        self.pending_literals.push(Literal::Boolean(value));
    }

    /// Handle a noun/pronoun/adjective - route to slot by case
    fn handle_nominal(&mut self, analysis: &MorphAnalysis, original: &str) -> Result<(), AssemblyError> {
        let constituent = Constituent {
            lemma: analysis.lemma.clone(),
            original: original.to_string(),
            case: analysis.case.unwrap_or(Case::Nominative),
            number: analysis.number,
            gender: analysis.gender,
        };

        match analysis.case {
            Some(Case::Nominative) => {
                if self.pending_subject.is_some() {
                    return Err(AssemblyError::DoubleSubject);
                }
                self.pending_subject = Some(constituent);
            }
            Some(Case::Accusative) => {
                if self.pending_object.is_some() {
                    return Err(AssemblyError::DoubleObject);
                }
                self.pending_object = Some(constituent);
            }
            Some(Case::Dative) => {
                // Dative can stack (multiple recipients) but for simplicity, one for now
                self.pending_indirect = Some(constituent);
            }
            Some(Case::Genitive) => {
                // Genitives attach to other constituents (possession, etc.)
                self.pending_genitives.push(constituent);
            }
            Some(Case::Vocative) => {
                // Vocative is direct address - treat as subject for now
                if self.pending_subject.is_none() {
                    self.pending_subject = Some(constituent);
                }
            }
            None => {
                // Unknown case - try to infer from context
                // Default to accusative (object) if we have no object
                if self.pending_object.is_none() {
                    self.pending_object = Some(constituent);
                }
            }
        }

        Ok(())
    }

    /// Handle a verb
    fn handle_verb(&mut self, analysis: &MorphAnalysis, original: &str) -> Result<(), AssemblyError> {
        if self.pending_verb.is_some() {
            return Err(AssemblyError::DoubleVerb);
        }

        self.pending_verb = Some(VerbConstituent {
            lemma: analysis.lemma.clone(),
            original: original.to_string(),
            person: analysis.person,
            number: analysis.number,
            tense: analysis.tense,
            mood: analysis.mood,
        });

        Ok(())
    }

    /// Finalize the statement - check agreement and assemble
    pub fn finalize(&mut self) -> Result<AssembledStatement, AssemblyError> {
        // Check for required verb (unless it's a query or has only literals)
        let has_content = self.pending_subject.is_some()
            || self.pending_object.is_some()
            || !self.pending_literals.is_empty();

        if self.pending_verb.is_none() && has_content && !self.is_query {
            // Allow verbless statements for queries and pure literal expressions
            // But for now, let's be lenient
        }

        // Check subject-verb agreement if both present
        if let (Some(subject), Some(verb)) = (&self.pending_subject, &self.pending_verb) {
            // In Greek, 3rd person subjects agree with 3rd person verbs
            // 1st/2nd person verbs often don't have explicit subjects (pro-drop)
            if let (Some(verb_person), Some(verb_number)) = (verb.person, verb.number) {
                if let Some(subj_number) = subject.number {
                    // Special rule: Neuter plural nouns take singular verbs in Greek!
                    let is_neuter_plural = subject.gender == Some(Gender::Neuter)
                        && subj_number == Number::Plural;

                    if !is_neuter_plural && subj_number != verb_number {
                        return Err(AssemblyError::SubjectVerbDisagreement {
                            subject: (Some(Person::Third), Some(subj_number)),
                            verb: (Some(verb_person), Some(verb_number)),
                        });
                    }
                }
            }
        }

        // Assemble the statement
        let statement = AssembledStatement {
            subject: self.pending_subject.take(),
            verb: self.pending_verb.take(),
            object: self.pending_object.take(),
            indirect: self.pending_indirect.take(),
            genitives: std::mem::take(&mut self.pending_genitives),
            literals: std::mem::take(&mut self.pending_literals),
            is_query: self.is_query,
        };

        self.reset();
        Ok(statement)
    }

    /// Check if the assembler has any pending content
    pub fn has_content(&self) -> bool {
        self.pending_subject.is_some()
            || self.pending_object.is_some()
            || self.pending_indirect.is_some()
            || self.pending_verb.is_some()
            || !self.pending_genitives.is_empty()
            || !self.pending_literals.is_empty()
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphology::analyze;

    #[test]
    fn test_simple_sov() {
        // ὁ ἄνθρωπος τὸν λόγον λέγει (The man says the word)
        let mut asm = Assembler::new();

        // Feed subject (nominative)
        let subj = analyze("ανθρωπος");
        asm.feed(&subj, "ἄνθρωπος").unwrap();

        // Feed object (accusative)
        let obj = analyze("λογον");
        asm.feed(&obj, "λόγον").unwrap();

        // Feed verb
        let verb = analyze("λεγει");
        asm.feed(&verb, "λέγει").unwrap();

        let stmt = asm.finalize().unwrap();
        assert!(stmt.subject.is_some());
        assert!(stmt.object.is_some());
        assert!(stmt.verb.is_some());
    }

    #[test]
    fn test_vso_same_result() {
        // λέγει τὸν λόγον ὁ ἄνθρωπος (VSO - same meaning)
        let mut asm = Assembler::new();

        // Feed verb first
        let verb = analyze("λεγει");
        asm.feed(&verb, "λέγει").unwrap();

        // Feed object
        let obj = analyze("λογον");
        asm.feed(&obj, "λόγον").unwrap();

        // Feed subject
        let subj = analyze("ανθρωπος");
        asm.feed(&subj, "ἄνθρωπος").unwrap();

        let stmt = asm.finalize().unwrap();
        assert!(stmt.subject.is_some());
        assert!(stmt.object.is_some());
        assert!(stmt.verb.is_some());
    }

    #[test]
    fn test_double_subject_error() {
        let mut asm = Assembler::new();

        let subj1 = analyze("ανθρωπος");
        asm.feed(&subj1, "ἄνθρωπος").unwrap();

        let subj2 = analyze("θεος");
        let result = asm.feed(&subj2, "θεός");

        assert!(matches!(result, Err(AssemblyError::DoubleSubject)));
    }

    #[test]
    fn test_double_verb_error() {
        let mut asm = Assembler::new();

        let verb1 = analyze("λεγει");
        asm.feed(&verb1, "λέγει").unwrap();

        let verb2 = analyze("γραφει");
        let result = asm.feed(&verb2, "γράφει");

        assert!(matches!(result, Err(AssemblyError::DoubleVerb)));
    }

    #[test]
    fn test_literals() {
        let mut asm = Assembler::new();

        asm.feed_string("χαῖρε κόσμε".to_string());

        let verb = analyze("λεγε");
        asm.feed(&verb, "λέγε").unwrap();

        let stmt = asm.finalize().unwrap();
        assert_eq!(stmt.literals.len(), 1);
        assert!(matches!(&stmt.literals[0], Literal::String(s) if s == "χαῖρε κόσμε"));
    }

    #[test]
    fn test_genitive_possession() {
        let mut asm = Assembler::new();

        // χρήστου ὄνομα (the name of the user)
        let genitive = analyze("χρηστου");
        asm.feed(&genitive, "χρήστου").unwrap();

        let nom = analyze("ονομα");
        asm.feed(&nom, "ὄνομα").unwrap();

        let stmt = asm.finalize().unwrap();
        assert_eq!(stmt.genitives.len(), 1);
        assert!(stmt.subject.is_some() || stmt.object.is_some());
    }

    #[test]
    fn test_dative_indirect_object() {
        let mut asm = Assembler::new();

        // τῷ ἀνθρώπῳ δίδωμι (I give to the man)
        let dat = analyze("ανθρωπω");
        asm.feed(&dat, "ἀνθρώπῳ").unwrap();

        let verb = analyze("διδωμι");
        asm.feed(&verb, "δίδωμι").unwrap();

        let stmt = asm.finalize().unwrap();
        assert!(stmt.indirect.is_some());
    }
}
