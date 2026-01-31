//! Expression state management
//!
//! Handles the accumulation of expression components like literals, operators,
//! and nested structures that are not strictly part of the sentence grammar.

use super::model::{Literal, ParticipleConstituent};
use crate::ast::{Expr, Word};
use crate::morphology::lexicon::BinaryOp;

/// Manages the state of expressions and literals
#[derive(Debug, Clone)]
pub struct ExpressionState {
    /// Accumulated literals (numbers, strings)
    pub pending_literals: Vec<Literal>,
    /// Accumulated array literals
    pub pending_arrays: Vec<Vec<Expr>>,
    pub pending_index_accesses: Vec<(Expr, Expr)>,
    pub pending_property_accesses: Vec<(String, String)>,
    pub pending_operators: Vec<BinaryOp>,
    pub pending_blocks: Vec<Vec<crate::ast::Statement>>,
    pub pending_nested_phrases: Vec<Vec<Expr>>,
    pub pending_participles: Vec<ParticipleConstituent>,
    pub pending_unwraps: Vec<Expr>,
}

impl ExpressionState {
    pub fn new() -> Self {
        Self {
            pending_literals: Vec::new(),
            pending_arrays: Vec::new(),
            pending_index_accesses: Vec::new(),
            pending_property_accesses: Vec::new(),
            pending_operators: Vec::new(),
            pending_blocks: Vec::new(),
            pending_nested_phrases: Vec::new(),
            pending_participles: Vec::new(),
            pending_unwraps: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.pending_literals.clear();
        self.pending_arrays.clear();
        self.pending_index_accesses.clear();
        self.pending_property_accesses.clear();
        self.pending_operators.clear();
        self.pending_blocks.clear();
        self.pending_nested_phrases.clear();
        self.pending_participles.clear();
        self.pending_unwraps.clear();
    }

    pub fn has_content(&self) -> bool {
        !self.pending_literals.is_empty()
            || !self.pending_arrays.is_empty()
            || !self.pending_index_accesses.is_empty()
            || !self.pending_property_accesses.is_empty()
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

    /// Feed an array literal
    pub fn feed_array(&mut self, elements: Vec<Expr>) {
        self.pending_arrays.push(elements);
    }

    /// Feed a parenthesized block (nested expression)
    pub fn feed_block(&mut self, statements: Vec<crate::ast::Statement>) {
        self.pending_blocks.push(statements);
    }

    /// Feed a nested phrase (parenthesized function call)
    pub fn feed_nested_phrase(&mut self, terms: Vec<Expr>) {
        self.pending_nested_phrases.push(terms);
    }

    /// Feed an index access (`array[index]`)
    pub fn feed_index_access(&mut self, array: Expr, index: Expr) {
        self.pending_index_accesses.push((array, index));
    }

    /// Feed an unwrap expression (expr!)
    pub fn feed_unwrap(&mut self, expr: Expr) {
        self.pending_unwraps.push(expr);
    }

    /// Feed a participle (for lambda construction)
    pub fn feed_participle(
        &mut self,
        analysis: &crate::morphology::ParticipleAnalysis,
        original: &str,
    ) {
        let constituent = ParticipleConstituent {
            verb_lemma: analysis.verb_lemma(),
            original: original.to_string(),
            tense: analysis.tense,
            voice: analysis.voice,
            case: analysis.case,
            gender: analysis.gender,
            number: analysis.number,
        };
        self.pending_participles.push(constituent);
    }

    /// Check for operators (boolean, comparison, arithmetic)
    pub fn check_operators(&mut self, normalized: &str, original: &str) -> bool {
        // Boolean operators
        if matches!(original, "καί" | "και") {
            self.pending_operators.push(BinaryOp::And);
            return true;
        }
        if matches!(original, "ἤ" | "ή") {
            // ἤ with breathing+accent, but not ᾖ
            self.pending_operators.push(BinaryOp::Or);
            return true;
        }

        // Comparison operators
        if let Some(op) = crate::morphology::lexicon::comparison_operator(normalized) {
            self.pending_operators.push(op);
            return true;
        }

        // Arithmetic operators
        if let Some(op) = crate::morphology::lexicon::arithmetic_operator(normalized) {
            self.pending_operators.push(op);
            return true;
        }

        false
    }

    /// Check for special properties (numerals, length, ordinals)
    /// Returns Some(result) if handled, where result indicates if subject should be consumed
    /// This method needs external coordination for subject consumption
    pub fn check_special_properties(
        &mut self,
        normalized: &str,
        subject_original: Option<&str>,
    ) -> (bool, bool) {
        // (handled, consume_subject)

        // Numeral words
        if let Some(value) = crate::morphology::lexicon::numeral_value(normalized) {
            self.pending_literals.push(Literal::Number(value));
            return (true, false);
        }

        // Property nouns (μῆκος)
        if crate::morphology::lexicon::is_length_property(normalized) {
            // If we have a subject, create a property access (use normalized original, not lemma)
            if let Some(subj_original) = subject_original {
                let normalized_original = crate::grammar::normalize_greek(subj_original);
                self.pending_property_accesses
                    .push((normalized_original, "len".to_string()));
                return (true, true); // Consume subject
            }
            return (true, false);
        }

        // Ordinal adjectives
        if crate::morphology::lexicon::is_ordinal(normalized) {
            // If we have a subject, create an index access with the ordinal index
            if let Some(subj_original) = subject_original
                && let Some(index) = crate::morphology::lexicon::ordinal_to_index(normalized)
            {
                // Create array and index expressions (use normalized original, not lemma)
                let normalized_original = crate::grammar::normalize_greek(subj_original);
                let array = Expr::Word(Word {
                    original: subj_original.to_string(),
                    normalized: normalized_original,
                });
                let index_expr = Expr::NumberLiteral(index);

                self.pending_index_accesses.push((array, index_expr));
                return (true, true); // Consume subject
            }
            return (true, false);
        }

        (false, false)
    }
}

impl Default for ExpressionState {
    fn default() -> Self {
        Self::new()
    }
}
