//! Sentence grammatical state management
//!
//! Handles the assignment of words to grammatical slots (Subject, Object, Verb)
//! based on case endings.

use super::model::{AssemblyError, Constituent, VerbConstituent};
use crate::morphology::{Case, MorphAnalysis};

/// Manages the grammatical state of the sentence (slots)
#[derive(Debug, Clone)]
pub struct SentenceState {
    /// Slot for the subject (Nominative case)
    pub pending_subject: Option<Constituent>,
    /// Storage for extra nominatives (e.g. predicate nominatives)
    pub pending_nominatives: Vec<Constituent>,
    /// Slot for the direct object (Accusative case)
    pub pending_object: Option<Constituent>,
    /// Slot for the indirect object (Dative case)
    pub pending_indirect: Option<Constituent>,
    /// Slot for the main verb
    pub pending_verb: Option<VerbConstituent>,
    /// Accumulated genitives (possessors)
    pub pending_genitives: Vec<Constituent>,
    /// Accumulated adjectives
    pub pending_adjectives: Vec<Constituent>,
}

impl SentenceState {
    pub fn new() -> Self {
        Self {
            pending_subject: None,
            pending_nominatives: Vec::new(),
            pending_object: None,
            pending_indirect: None,
            pending_verb: None,
            pending_genitives: Vec::new(),
            pending_adjectives: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.pending_subject = None;
        self.pending_nominatives.clear();
        self.pending_object = None;
        self.pending_indirect = None;
        self.pending_verb = None;
        self.pending_genitives.clear();
        self.pending_adjectives.clear();
    }

    pub fn has_content(&self) -> bool {
        self.pending_subject.is_some()
            || self.pending_object.is_some()
            || self.pending_indirect.is_some()
            || self.pending_verb.is_some()
            || !self.pending_genitives.is_empty()
            || !self.pending_adjectives.is_empty()
    }

    /// Handle a noun/pronoun/adjective - route to slot by case
    pub fn handle_nominal(
        &mut self,
        analysis: &MorphAnalysis,
        original: &str,
    ) -> Result<(), AssemblyError> {
        let constituent = Constituent {
            lemma: analysis.lemma.to_string(),
            original: original.to_string(),
            case: analysis.case.unwrap_or(Case::Nominative),
            number: analysis.number,
            gender: analysis.gender,
        };

        match analysis.case {
            Some(Case::Nominative) => {
                if self.pending_subject.is_some() {
                    // Additional nominatives stored separately for function call patterns
                    self.pending_nominatives.push(constituent);
                } else {
                    self.pending_subject = Some(constituent);
                }
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
    pub fn handle_verb(
        &mut self,
        analysis: &MorphAnalysis,
        original: &str,
    ) -> Result<(), AssemblyError> {
        if self.pending_verb.is_some() {
            return Err(AssemblyError::DoubleVerb);
        }

        self.pending_verb = Some(VerbConstituent {
            lemma: analysis.lemma.to_string(),
            original: original.to_string(),
            person: analysis.person,
            number: analysis.number,
            tense: analysis.tense,
            mood: analysis.mood,
            voice: analysis.voice,
        });

        Ok(())
    }

    /// Handle an adjective - store it for later pattern matching
    pub fn handle_adjective(
        &mut self,
        analysis: &MorphAnalysis,
        original: &str,
    ) -> Result<(), AssemblyError> {
        let constituent = Constituent {
            lemma: analysis.lemma.to_string(),
            original: original.to_string(),
            case: analysis.case.unwrap_or(Case::Nominative),
            number: analysis.number,
            gender: analysis.gender,
        };

        self.pending_adjectives.push(constituent);
        Ok(())
    }
}

impl Default for SentenceState {
    fn default() -> Self {
        Self::new()
    }
}
