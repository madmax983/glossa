//! Core Assembler implementation
//!
//! This module contains the `Assembler` struct and its primary public API.
//! It orchestrates the assembly process by delegating specific tasks to
//! handlers and check modules.

use crate::ast::Expr;
use crate::errors::AssemblyError;
use crate::morphology::{MorphAnalysis, PartOfSpeech};
use crate::semantic::assembly::checks;
use crate::semantic::assembly::handlers;
use crate::semantic::assembly::model::{
    AssembledStatement, Literal, MAX_ARRAYS, MAX_BLOCKS, MAX_INDEX_ACCESSES, MAX_LITERALS,
    MAX_NESTED_PHRASES, MAX_PARTICIPLES, MAX_UNWRAPS, ParticipleConstituent,
};
use crate::text::normalize_greek;
use smol_str::SmolStr;

/// The slot-based assembler
///
/// Feed it tokens one by one, and it routes them to the appropriate slot
/// based on their grammatical case. When you hit end-of-statement, call
/// `finalize()` to get the assembled statement.
///
/// # State Machine
///
/// The assembler maintains "pending" slots in its `state`. As tokens arrive:
/// - **Nominative** -> `state.subject` (or `state.nominatives` if subject full)
/// - **Accusative** -> `state.object`
/// - **Dative** -> `state.indirect`
/// - **Verb** -> `state.verb`
///
/// This allows tokens to arrive in any order (Subject-Verb-Object, Verb-Object-Subject, etc.)
/// and still fill the correct semantic roles.
pub struct Assembler {
    pub(crate) state: AssembledStatement,
}

impl Assembler {
    /// Create a new empty assembler
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use glossa::semantic::Assembler;
    ///
    /// let asm = Assembler::new();
    /// ```
    pub fn new() -> Self {
        Assembler {
            state: AssembledStatement::default(),
        }
    }

    /// Mark this statement as a query
    pub fn set_query(&mut self, is_query: bool) {
        self.state.is_query = is_query;
    }

    /// Mark this statement as propagation (ends with `;` → converts to `?`)
    pub fn set_propagate(&mut self, is_propagate: bool) {
        self.state.is_propagate = is_propagate;
    }

    /// Feed a morphologically-analyzed token into the assembler
    ///
    /// This routes the token to the correct slot based on its case.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use glossa::semantic::Assembler;
    /// use glossa::morphology::analyze;
    ///
    /// let mut asm = Assembler::new();
    ///
    /// // "ἄνθρωπος" (Nom) -> Subject
    /// let subj = analyze("ἄνθρωπος");
    /// asm.feed(&subj, "ἄνθρωπος").unwrap();
    ///
    /// // "λόγον" (Acc) -> Object
    /// let obj = analyze("λόγον");
    /// asm.feed(&obj, "λόγον").unwrap();
    /// ```
    #[allow(dead_code)]
    pub fn feed(&mut self, analysis: &MorphAnalysis, original: &str) -> Result<(), AssemblyError> {
        let normalized = normalize_greek(original);
        self.feed_with_normalized(analysis, original, &normalized)
    }

    /// Feed a morphologically-analyzed token with pre-computed normalization
    ///
    /// This is a zero-allocation path when the normalized form is already known (e.g. from AST).
    /// It bypasses the costly `normalize_greek` call which may allocate strings.
    pub fn feed_with_normalized(
        &mut self,
        analysis: &MorphAnalysis,
        original: &str,
        normalized: &str,
    ) -> Result<(), AssemblyError> {
        if checks::check_special_markers(&mut self.state, normalized, original) {
            return Ok(());
        }

        if checks::check_method_verbs(&mut self.state, normalized)? {
            return Ok(());
        }

        if checks::check_operators(&mut self.state, normalized, original)? {
            return Ok(());
        }

        if checks::check_special_properties(&mut self.state, normalized)? {
            return Ok(());
        }

        match analysis.part_of_speech {
            PartOfSpeech::Noun | PartOfSpeech::Pronoun => {
                handlers::handle_nominal(&mut self.state, analysis, original, normalized)
            }
            PartOfSpeech::Adjective => {
                handlers::handle_adjective(&mut self.state, analysis, original, normalized)
            }
            PartOfSpeech::Verb => {
                handlers::handle_verb(&mut self.state, analysis, original, normalized)
            }
            PartOfSpeech::Numeral => {
                // Already handled above, but keep this for explicit numeral POS
                handlers::handle_nominal(&mut self.state, analysis, original, normalized)
            }
            PartOfSpeech::Conjunction => {
                // Non-operator conjunctions are ignored for now
                Ok(())
            }
            _ => Ok(()), // Ignore particles, articles for now
        }
    }

    /// Feed a string literal
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use glossa::semantic::Assembler;
    ///
    /// let mut asm = Assembler::new();
    /// asm.feed_string("χαῖρε".to_string()).unwrap();
    /// ```
    pub fn feed_string(&mut self, value: String) -> Result<(), AssemblyError> {
        if self.state.literals.len() >= MAX_LITERALS {
            return Err(AssemblyError::LimitExceeded {
                resource: "Literals".to_string(),
                max: MAX_LITERALS,
            });
        }
        self.state.literals.push(Literal::String(value));
        Ok(())
    }

    /// Feed a number literal
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use glossa::semantic::Assembler;
    ///
    /// let mut asm = Assembler::new();
    /// asm.feed_number(42).unwrap();
    /// ```
    pub fn feed_number(&mut self, value: i64) -> Result<(), AssemblyError> {
        if self.state.literals.len() >= MAX_LITERALS {
            return Err(AssemblyError::LimitExceeded {
                resource: "Literals".to_string(),
                max: MAX_LITERALS,
            });
        }
        self.state.literals.push(Literal::Number(value));
        Ok(())
    }

    /// Feed a boolean literal
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use glossa::semantic::Assembler;
    ///
    /// let mut asm = Assembler::new();
    /// asm.feed_boolean(true).unwrap();
    /// ```
    pub fn feed_boolean(&mut self, value: bool) -> Result<(), AssemblyError> {
        if self.state.literals.len() >= MAX_LITERALS {
            return Err(AssemblyError::LimitExceeded {
                resource: "Literals".to_string(),
                max: MAX_LITERALS,
            });
        }
        self.state.literals.push(Literal::Boolean(value));
        Ok(())
    }

    /// Feed an array literal
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use glossa::semantic::Assembler;
    /// use glossa::ast::{Expr, Word};
    ///
    /// let mut asm = Assembler::new();
    /// let elements = vec![Expr::NumberLiteral(1), Expr::NumberLiteral(2)];
    /// asm.feed_array(elements).unwrap();
    /// ```
    pub fn feed_array(&mut self, elements: Vec<Expr>) -> Result<(), AssemblyError> {
        if self.state.arrays.len() >= MAX_ARRAYS {
            return Err(AssemblyError::LimitExceeded {
                resource: "Arrays".to_string(),
                max: MAX_ARRAYS,
            });
        }
        self.state.arrays.push(elements);
        Ok(())
    }

    /// Feed a parenthesized block (nested expression)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use glossa::semantic::Assembler;
    ///
    /// let mut asm = Assembler::new();
    /// asm.feed_block(vec![]).unwrap(); // Empty block
    /// ```
    pub fn feed_block(
        &mut self,
        statements: Vec<crate::ast::Statement>,
    ) -> Result<(), AssemblyError> {
        if self.state.blocks.len() >= MAX_BLOCKS {
            return Err(AssemblyError::LimitExceeded {
                resource: "Blocks".to_string(),
                max: MAX_BLOCKS,
            });
        }
        self.state.blocks.push(statements);
        Ok(())
    }

    /// Feed a nested phrase (parenthesized function call)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use glossa::semantic::Assembler;
    /// use glossa::ast::Expr;
    ///
    /// let mut asm = Assembler::new();
    /// asm.feed_nested_phrase(vec![Expr::NumberLiteral(1)]).unwrap();
    /// ```
    pub fn feed_nested_phrase(&mut self, terms: Vec<Expr>) -> Result<(), AssemblyError> {
        if self.state.nested_phrases.len() >= MAX_NESTED_PHRASES {
            return Err(AssemblyError::LimitExceeded {
                resource: "Nested Phrases".to_string(),
                max: MAX_NESTED_PHRASES,
            });
        }
        self.state.nested_phrases.push(terms);
        Ok(())
    }

    /// Feed an index access (`array[index]`)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use glossa::semantic::Assembler;
    /// use glossa::ast::{Expr, Word};
    ///
    /// let mut asm = Assembler::new();
    /// let array = Expr::Word(Word {
    ///     original: "πίναξ".into(),
    ///     normalized: "πιναξ".into(),
    /// });
    /// let index = Expr::NumberLiteral(0);
    /// asm.feed_index_access(array, index).unwrap();
    /// ```
    pub fn feed_index_access(&mut self, array: Expr, index: Expr) -> Result<(), AssemblyError> {
        if self.state.index_accesses.len() >= MAX_INDEX_ACCESSES {
            return Err(AssemblyError::LimitExceeded {
                resource: "Index Accesses".to_string(),
                max: MAX_INDEX_ACCESSES,
            });
        }
        self.state.index_accesses.push((array, index));
        Ok(())
    }

    /// Feed an unwrap expression (expr!)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use glossa::semantic::Assembler;
    /// use glossa::ast::{Expr, Word};
    ///
    /// let mut asm = Assembler::new();
    /// let expr = Expr::Word(Word {
    ///     original: "τιμή".into(),
    ///     normalized: "τιμη".into(),
    /// });
    /// asm.feed_unwrap(expr).unwrap();
    /// ```
    pub fn feed_unwrap(&mut self, expr: Expr) -> Result<(), AssemblyError> {
        if self.state.unwraps.len() >= MAX_UNWRAPS {
            return Err(AssemblyError::LimitExceeded {
                resource: "Unwraps".to_string(),
                max: MAX_UNWRAPS,
            });
        }
        self.state.unwraps.push(expr);
        Ok(())
    }

    /// Feed a participle (for lambda construction)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use glossa::semantic::Assembler;
    /// use glossa::morphology::{ParticipleAnalysis, Tense, Voice, Case, Gender, Number};
    ///
    /// let mut asm = Assembler::new();
    /// let analysis = ParticipleAnalysis {
    ///     stem: "διπλασιαζ".to_string(),
    ///     tense: Tense::Present,
    ///     voice: Voice::Middle,
    ///     case: Case::Nominative,
    ///     gender: Gender::Neuter,
    ///     number: Number::Plural,
    ///     confidence: 1.0,
    /// };
    /// asm.feed_participle(&analysis, "διπλασιαζόμενα").unwrap();
    /// ```
    pub fn feed_participle(
        &mut self,
        analysis: &crate::morphology::ParticipleAnalysis,
        original: &str,
    ) -> Result<(), AssemblyError> {
        if self.state.participles.len() >= MAX_PARTICIPLES {
            return Err(AssemblyError::LimitExceeded {
                resource: "Participles".to_string(),
                max: MAX_PARTICIPLES,
            });
        }
        let normalized = normalize_greek(original);
        let constituent = ParticipleConstituent {
            // OPTIMIZATION: verb_lemma() returns a normalized string
            verb_lemma: SmolStr::new(analysis.verb_lemma()),
            original: original.into(),
            normalized,
            tense: analysis.tense,
            voice: analysis.voice,
            case: analysis.case,
            gender: analysis.gender,
            number: analysis.number,
        };
        self.state.participles.push(constituent);
        Ok(())
    }

    /// Finalize the statement - check agreement and assemble
    ///
    /// This validates the sentence structure (e.g. subject-verb agreement) and
    /// returns the complete `AssembledStatement`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use glossa::semantic::Assembler;
    /// use glossa::morphology::analyze;
    ///
    /// let mut asm = Assembler::new();
    ///
    /// // "The man says"
    /// asm.feed(&analyze("ἄνθρωπος"), "ἄνθρωπος").unwrap();
    /// asm.feed(&analyze("λέγει"), "λέγει").unwrap();
    ///
    /// let stmt = asm.finalize().unwrap();
    /// assert!(stmt.subject.is_some());
    /// assert!(stmt.verb.is_some());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Two subjects or two verbs are found (`DoubleSubject`, `DoubleVerb`)
    /// - Subject and verb disagree in number (`SubjectVerbDisagreement`)
    /// - Grammatical gender mismatch occurs
    pub fn finalize(&mut self) -> Result<AssembledStatement, AssemblyError> {
        // Check for required verb (unless it's a query or has only literals)
        let has_content = self.state.subject.is_some()
            || self.state.object.is_some()
            || !self.state.literals.is_empty();

        if self.state.verb.is_none() && has_content && !self.state.is_query {
            // Allow verbless statements for queries and pure literal expressions
            // But for now, let's be lenient
        }

        // Check subject-verb agreement if both present
        if let (Some(subject), Some(verb)) = (&self.state.subject, &self.state.verb) {
            checks::check_agreement(subject, verb)?;
        }

        // Return the assembled statement
        let statement = std::mem::take(&mut self.state);
        Ok(statement)
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}
