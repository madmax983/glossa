use super::state::{
    AssembledStatement, Constituent, Literal, MAX_INDEX_ACCESSES, MAX_OPERATORS,
    MAX_PROPERTY_ACCESSES, VerbConstituent,
};
use crate::ast::{Expr, Word};
use crate::errors::assembly::AssemblyError;
use crate::morphology::lexicon::BinaryOp;
use crate::morphology::{Gender, Mood, Number, Person};
use crate::text::normalize_greek;
use unicode_normalization::UnicodeNormalization;

/// Check for special markers (mutable, containment, delimiter)
pub fn check_special_markers(
    state: &mut AssembledStatement,
    normalized: &str,
    original: &str,
) -> bool {
    // Check for mutable marker (μετά)
    if crate::morphology::lexicon::is_mutable_marker(normalized) {
        state.has_mutable_marker = true;
        return true;
    }

    // Check for containment preposition (ἐν)
    if crate::morphology::lexicon::is_containment_preposition(normalized) {
        // DISAMBIGUATION: ἐν (in) vs ἕν (one)
        // If original has rough breathing (U+0314 combining reversed comma above), it's "one".
        // We check the NFD form to separate base letters from diacritics.
        if original.nfd().any(|c| c == '\u{0314}') {
            return false;
        }

        state.has_containment_preposition = true;
        return true;
    }

    // Check for delimiter preposition (κατά)
    if crate::morphology::lexicon::is_delimiter_preposition(normalized) {
        state.has_delimiter_preposition = true;
        return true;
    }

    false
}

/// Check for method verbs (split, join)
pub fn check_method_verbs(state: &mut AssembledStatement, normalized: &str) -> bool {
    // Check for split verb
    if crate::morphology::lexicon::is_split_verb(normalized)
        && try_create_string_method(state, "split")
    {
        return true;
    }

    // Check for join verb
    if crate::morphology::lexicon::is_join_verb(normalized)
        && try_create_string_method(state, "join")
    {
        return true;
    }

    false
}

/// Helper to create a string method call if conditions are met
#[allow(clippy::collapsible_if)]
fn try_create_string_method(state: &mut AssembledStatement, method_name: &str) -> bool {
    if state.has_delimiter_preposition && matches!(state.literals.last(), Some(Literal::String(_)))
    {
        if let Some(ref subj) = state.subject {
            if state.property_accesses.len() >= MAX_PROPERTY_ACCESSES {
                return false;
            }

            let delim = match state.literals.pop() {
                Some(Literal::String(s)) => s,
                _ => unreachable!(),
            };

            let normalized_original = normalize_greek(&subj.original);
            state.string_method = Some((method_name.to_string(), delim));
            state
                .property_accesses
                .push((normalized_original.to_string(), method_name.to_string()));
            return true;
        }
    }
    false
}

/// Check for operators (boolean, comparison, arithmetic)
pub fn check_operators(state: &mut AssembledStatement, normalized: &str, original: &str) -> bool {
    // Boolean operators
    if matches!(original, "καί" | "και") {
        if state.operators.len() >= MAX_OPERATORS {
            return false;
        }
        state.operators.push(BinaryOp::And);
        return true;
    }
    if matches!(original, "ἤ" | "ή") {
        if state.operators.len() >= MAX_OPERATORS {
            return false;
        }
        // ἤ with breathing+accent, but not ᾖ
        state.operators.push(BinaryOp::Or);
        return true;
    }

    // Comparison operators
    if let Some(op) = crate::morphology::lexicon::comparison_operator(normalized) {
        if state.operators.len() >= MAX_OPERATORS {
            return false;
        }
        state.operators.push(op);
        return true;
    }

    // Arithmetic operators
    if let Some(op) = crate::morphology::lexicon::arithmetic_operator(normalized) {
        if state.operators.len() >= MAX_OPERATORS {
            return false;
        }
        state.operators.push(op);
        return true;
    }

    false
}

/// Check for special properties (numerals, length, ordinals)
pub fn check_special_properties(state: &mut AssembledStatement, normalized: &str) -> bool {
    // Numeral words
    if let Some(value) = crate::morphology::lexicon::numeral_value(normalized) {
        state.literals.push(Literal::Number(value));
        return true;
    }

    // Property nouns (μῆκος)
    if crate::morphology::lexicon::is_length_property(normalized) {
        // If we have a subject, create a property access (use normalized original, not lemma)
        if let Some(ref subj) = state.subject {
            if state.property_accesses.len() >= MAX_PROPERTY_ACCESSES {
                return false;
            }
            let normalized_original = crate::text::normalize_greek(&subj.original);
            state
                .property_accesses
                .push((normalized_original.to_string(), "len".to_string()));
            state.subject = None; // Consume the subject
            return true;
        }
    }

    // Ordinal adjectives
    if crate::morphology::lexicon::is_ordinal(normalized) {
        // If we have a subject, create an index access with the ordinal index
        if let Some(ref subj) = state.subject
            && let Some(index) = crate::morphology::lexicon::ordinal_to_index(normalized)
        {
            if state.index_accesses.len() >= MAX_INDEX_ACCESSES {
                return false;
            }
            // Create array and index expressions (use normalized original, not lemma)
            let normalized_original = crate::text::normalize_greek(&subj.original);
            let array = Expr::Word(Word {
                original: subj.original.clone(),
                normalized: normalized_original.clone(),
            });
            let index_expr = Expr::NumberLiteral(index);

            state.index_accesses.push((array, index_expr));
            state.subject = None; // Consume the subject
            return true;
        }
    }

    false
}

/// Check subject-verb agreement
pub fn check_agreement(subject: &Constituent, verb: &VerbConstituent) -> Result<(), AssemblyError> {
    if let (Some(verb_person), Some(verb_number), Some(subj_number)) =
        (verb.person, verb.number, subject.number)
    {
        // Determine subject person (default to 3rd for nouns if not specified)
        let subj_person = subject.person.unwrap_or(Person::Third);

        // Check person agreement
        // Exception: Allow Imperative verbs to disagree (e.g. "User, print!" uses 2nd person verb with 3rd person subject)
        let is_imperative = verb.mood == Some(Mood::Imperative);
        if !is_imperative && subj_person != verb_person {
            return Err(AssemblyError::SubjectVerbDisagreement {
                subject: (Some(subj_person), Some(subj_number)),
                verb: (Some(verb_person), Some(verb_number)),
            });
        }

        // Special rule: Neuter plural nouns take singular verbs in Greek!
        let is_neuter_plural =
            subject.gender == Some(Gender::Neuter) && subj_number == Number::Plural;

        if !is_neuter_plural && subj_number != verb_number {
            return Err(AssemblyError::SubjectVerbDisagreement {
                subject: (Some(subj_person), Some(subj_number)),
                verb: (Some(verb_person), Some(verb_number)),
            });
        }
    }
    Ok(())
}
