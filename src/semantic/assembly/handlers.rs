//! Logic for handling different parts of speech
//!
//! This module contains the functions that process nouns, verbs, adjectives,
//! and other constituents, routing them to the appropriate slots in the
//! `AssembledStatement` and performing necessary validations.

use crate::errors::AssemblyError;
use crate::morphology::{Case, MorphAnalysis};
use crate::semantic::assembly::checks::check_agreement;
use crate::semantic::assembly::model::{
    AssembledStatement, Constituent, MAX_ADJECTIVES, MAX_GENITIVES, MAX_NOMINATIVES,
    VerbConstituent,
};
use smol_str::SmolStr;

/// Handle a noun/pronoun/adjective - route to slot by case
pub(crate) fn handle_nominal(
    state: &mut AssembledStatement,
    analysis: &MorphAnalysis,
    original: &str,
    normalized: &str,
) -> Result<(), AssemblyError> {
    let constituent = Constituent {
        // OPTIMIZATION: Lemma is guaranteed to be normalized by morphology analysis
        lemma: SmolStr::new(analysis.lemma.as_ref()),
        original: original.into(),
        normalized: normalized.into(),
        case: analysis.case.unwrap_or(Case::Nominative),
        number: analysis.number,
        gender: analysis.gender,
        person: analysis.person,
    };

    match analysis.case {
        Some(Case::Nominative) => handle_nominative(state, constituent),
        Some(Case::Accusative) => handle_accusative(state, constituent),
        Some(Case::Dative) => handle_dative(state, constituent),
        Some(Case::Genitive) => handle_genitive(state, constituent),
        Some(Case::Vocative) => handle_vocative(state, constituent),
        None => handle_unknown_case(state, constituent),
    }
}

fn handle_nominative(
    state: &mut AssembledStatement,
    constituent: Constituent,
) -> Result<(), AssemblyError> {
    // If we already have a verb, check agreement immediately!
    if let Some(verb) = &state.verb {
        // Don't check agreement if we already have a subject (this is an extra nominative)
        if state.subject.is_none() {
            check_agreement(&constituent, verb)?;
        }
    }

    if state.subject.is_some() {
        // Additional nominatives stored separately for function call patterns
        if state.nominatives.len() >= MAX_NOMINATIVES {
            return Err(AssemblyError::LimitExceeded {
                resource: "Nominatives".to_string(),
                max: MAX_NOMINATIVES,
            });
        }
        state.nominatives.push(constituent);
    } else {
        state.subject = Some(constituent);
    }
    Ok(())
}

fn handle_accusative(
    state: &mut AssembledStatement,
    constituent: Constituent,
) -> Result<(), AssemblyError> {
    if state.object.is_some() {
        return Err(AssemblyError::DoubleObject);
    }
    state.object = Some(constituent);
    Ok(())
}

fn handle_dative(
    state: &mut AssembledStatement,
    constituent: Constituent,
) -> Result<(), AssemblyError> {
    // Dative can stack (multiple recipients) but for simplicity, one for now
    if state.indirect.is_some() {
        return Err(AssemblyError::DoubleIndirect);
    }
    state.indirect = Some(constituent);
    Ok(())
}

fn handle_genitive(
    state: &mut AssembledStatement,
    constituent: Constituent,
) -> Result<(), AssemblyError> {
    // Genitives attach to other constituents (possession, etc.)
    if state.genitives.len() >= MAX_GENITIVES {
        return Err(AssemblyError::LimitExceeded {
            resource: "Genitives".to_string(),
            max: MAX_GENITIVES,
        });
    }
    state.genitives.push(constituent);
    Ok(())
}

fn handle_vocative(
    state: &mut AssembledStatement,
    constituent: Constituent,
) -> Result<(), AssemblyError> {
    // Vocative is direct address - treat as subject for now
    if state.subject.is_none() {
        state.subject = Some(constituent);
    }
    Ok(())
}

fn handle_unknown_case(
    state: &mut AssembledStatement,
    constituent: Constituent,
) -> Result<(), AssemblyError> {
    // Unknown case - try to infer from context
    // Default to accusative (object) if we have no object
    if state.object.is_none() {
        state.object = Some(constituent);
        Ok(())
    } else {
        Err(AssemblyError::DoubleObject)
    }
}

/// Handle a verb
pub(crate) fn handle_verb(
    state: &mut AssembledStatement,
    analysis: &MorphAnalysis,
    original: &str,
    normalized: &str,
) -> Result<(), AssemblyError> {
    if state.verb.is_some() {
        return Err(AssemblyError::DoubleVerb);
    }

    let verb_constituent = VerbConstituent {
        // OPTIMIZATION: Lemma is guaranteed to be normalized by morphology analysis
        lemma: SmolStr::new(analysis.lemma.as_ref()),
        original: original.into(),
        normalized: normalized.into(),
        person: analysis.person,
        number: analysis.number,
        tense: analysis.tense,
        mood: analysis.mood,
        voice: analysis.voice,
    };

    // If we already have a subject, check agreement immediately!
    if let Some(subject) = &state.subject {
        check_agreement(subject, &verb_constituent)?;
    }

    state.verb = Some(verb_constituent);

    Ok(())
}

/// Handle an adjective - store it for later pattern matching
pub(crate) fn handle_adjective(
    state: &mut AssembledStatement,
    analysis: &MorphAnalysis,
    original: &str,
    normalized: &str,
) -> Result<(), AssemblyError> {
    let constituent = Constituent {
        // OPTIMIZATION: Lemma is guaranteed to be normalized by morphology analysis
        lemma: SmolStr::new(analysis.lemma.as_ref()),
        original: original.into(),
        normalized: normalized.into(),
        case: analysis.case.unwrap_or(Case::Nominative),
        number: analysis.number,
        gender: analysis.gender,
        person: None, // Adjectives don't really have person
    };

    if state.adjectives.len() >= MAX_ADJECTIVES {
        return Err(AssemblyError::LimitExceeded {
            resource: "Adjectives".to_string(),
            max: MAX_ADJECTIVES,
        });
    }
    state.adjectives.push(constituent);
    Ok(())
}
