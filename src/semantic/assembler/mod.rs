//! Facade for the semantic assembler
//!
//! Orchestrates the `SentenceState` (grammatical slots) and `ExpressionState` (AST components)
//! to assemble full statements from morphological tokens.

pub mod expression;
pub mod model;
pub mod sentence;

pub use model::{
    AssembledStatement, AssemblyError, Constituent, Literal, ParticipleConstituent, VerbConstituent,
};

use self::expression::ExpressionState;
use self::sentence::SentenceState;
use crate::ast::Expr;
use crate::grammar::normalize_greek;
use crate::morphology::{Gender, MorphAnalysis, Number, PartOfSpeech, Person};

/// The slot-based assembler
///
/// Feed it tokens one by one, and it routes them to the appropriate slot
/// based on their grammatical case. When you hit end-of-statement, call
/// `finalize()` to get the assembled statement.
pub struct Assembler {
    sentence: SentenceState,
    expression: ExpressionState,

    // Flags and special state
    pub is_query: bool,
    pub is_propagate: bool,
    pub pending_mutable_marker: bool,
    pub has_containment_preposition: bool,
    pub has_delimiter_preposition: bool,
    pub pending_string_method: Option<(String, String)>,
}

impl Assembler {
    /// Create a new empty assembler
    pub fn new() -> Self {
        Self {
            sentence: SentenceState::new(),
            expression: ExpressionState::new(),
            is_query: false,
            is_propagate: false,
            pending_mutable_marker: false,
            has_containment_preposition: false,
            has_delimiter_preposition: false,
            pending_string_method: None,
        }
    }

    /// Reset the assembler for a new statement
    pub fn reset(&mut self) {
        self.sentence.reset();
        self.expression.reset();
        self.is_query = false;
        self.is_propagate = false;
        self.pending_mutable_marker = false;
        self.has_containment_preposition = false;
        self.has_delimiter_preposition = false;
        self.pending_string_method = None;
    }

    /// Mark this statement as a query
    pub fn set_query(&mut self, is_query: bool) {
        self.is_query = is_query;
    }

    /// Mark this statement as propagation (ends with `;` → converts to `?`)
    pub fn set_propagate(&mut self, is_propagate: bool) {
        self.is_propagate = is_propagate;
    }

    /// Feed a morphologically-analyzed token into the assembler
    pub fn feed(&mut self, analysis: &MorphAnalysis, original: &str) -> Result<(), AssemblyError> {
        let normalized = normalize_greek(original);

        if self.check_special_markers(&normalized) {
            return Ok(());
        }

        if self.check_method_verbs(&normalized) {
            return Ok(());
        }

        if self.expression.check_operators(&normalized, original) {
            return Ok(());
        }

        let (handled, consume_subject) = self.expression.check_special_properties(
            &normalized,
            self.sentence
                .pending_subject
                .as_ref()
                .map(|s| s.original.as_str()),
        );

        if handled {
            if consume_subject {
                self.sentence.pending_subject = None;
            }
            return Ok(());
        }

        match analysis.part_of_speech {
            PartOfSpeech::Noun | PartOfSpeech::Pronoun => {
                self.sentence.handle_nominal(analysis, original)
            }
            PartOfSpeech::Adjective => self.sentence.handle_adjective(analysis, original),
            PartOfSpeech::Verb => self.sentence.handle_verb(analysis, original),
            PartOfSpeech::Numeral => {
                // Already handled above, but keep this for explicit numeral POS
                self.sentence.handle_nominal(analysis, original)
            }
            PartOfSpeech::Conjunction => {
                // Non-operator conjunctions are ignored for now
                Ok(())
            }
            _ => Ok(()), // Ignore particles, articles for now
        }
    }

    // Delegate feed methods to expression state

    pub fn feed_string(&mut self, value: String) {
        self.expression.feed_string(value);
    }

    pub fn feed_number(&mut self, value: i64) {
        self.expression.feed_number(value);
    }

    pub fn feed_boolean(&mut self, value: bool) {
        self.expression.feed_boolean(value);
    }

    pub fn feed_array(&mut self, elements: Vec<Expr>) {
        self.expression.feed_array(elements);
    }

    pub fn feed_block(&mut self, statements: Vec<crate::ast::Statement>) {
        self.expression.feed_block(statements);
    }

    pub fn feed_nested_phrase(&mut self, terms: Vec<Expr>) {
        self.expression.feed_nested_phrase(terms);
    }

    pub fn feed_index_access(&mut self, array: Expr, index: Expr) {
        self.expression.feed_index_access(array, index);
    }

    pub fn feed_unwrap(&mut self, expr: Expr) {
        self.expression.feed_unwrap(expr);
    }

    pub fn feed_participle(
        &mut self,
        analysis: &crate::morphology::ParticipleAnalysis,
        original: &str,
    ) {
        self.expression.feed_participle(analysis, original);
    }

    /// Finalize the statement - check agreement and assemble
    pub fn finalize(&mut self) -> Result<AssembledStatement, AssemblyError> {
        // Check for required verb (unless it's a query or has only literals)
        let has_content = self.sentence.has_content() || self.expression.has_content();

        if self.sentence.pending_verb.is_none() && has_content && !self.is_query {
            // Allow verbless statements for queries and pure literal expressions
            // But for now, let's be lenient
        }

        // Check subject-verb agreement if both present
        if let (Some(subject), Some(verb)) =
            (&self.sentence.pending_subject, &self.sentence.pending_verb)
        {
            // In Greek, 3rd person subjects agree with 3rd person verbs
            // 1st/2nd person verbs often don't have explicit subjects (pro-drop)
            if let (Some(verb_person), Some(verb_number)) = (verb.person, verb.number)
                && let Some(subj_number) = subject.number
            {
                // Special rule: Neuter plural nouns take singular verbs in Greek!
                let is_neuter_plural =
                    subject.gender == Some(Gender::Neuter) && subj_number == Number::Plural;

                if !is_neuter_plural && subj_number != verb_number {
                    return Err(AssemblyError::SubjectVerbDisagreement {
                        subject: (Some(Person::Third), Some(subj_number)),
                        verb: (Some(verb_person), Some(verb_number)),
                    });
                }
            }
        }

        // Assemble the statement
        let statement = AssembledStatement {
            subject: self.sentence.pending_subject.take(),
            nominatives: std::mem::take(&mut self.sentence.pending_nominatives),
            verb: self.sentence.pending_verb.take(),
            object: self.sentence.pending_object.take(),
            indirect: self.sentence.pending_indirect.take(),
            genitives: std::mem::take(&mut self.sentence.pending_genitives),
            adjectives: std::mem::take(&mut self.sentence.pending_adjectives),
            literals: std::mem::take(&mut self.expression.pending_literals),
            arrays: std::mem::take(&mut self.expression.pending_arrays),
            index_accesses: std::mem::take(&mut self.expression.pending_index_accesses),
            property_accesses: std::mem::take(&mut self.expression.pending_property_accesses),
            operators: std::mem::take(&mut self.expression.pending_operators),
            blocks: std::mem::take(&mut self.expression.pending_blocks),
            nested_phrases: std::mem::take(&mut self.expression.pending_nested_phrases),
            participles: std::mem::take(&mut self.expression.pending_participles),
            unwraps: std::mem::take(&mut self.expression.pending_unwraps),
            has_mutable_marker: self.pending_mutable_marker,
            is_query: self.is_query,
            is_propagate: self.is_propagate,
            has_containment_preposition: self.has_containment_preposition,
            has_delimiter_preposition: self.has_delimiter_preposition,
            string_method: self.pending_string_method.take(),
        };

        self.reset();
        Ok(statement)
    }

    /// Check if the assembler has any pending content
    pub fn has_content(&self) -> bool {
        self.sentence.has_content() || self.expression.has_content()
    }

    /// Check for special markers (mutable, containment, delimiter)
    fn check_special_markers(&mut self, normalized: &str) -> bool {
        // Check for mutable marker (μετά)
        if crate::morphology::lexicon::is_mutable_marker(normalized) {
            self.pending_mutable_marker = true;
            return true;
        }

        // Check for containment preposition (ἐν)
        if crate::morphology::lexicon::is_containment_preposition(normalized) {
            self.has_containment_preposition = true;
            return true;
        }

        // Check for delimiter preposition (κατά)
        if crate::morphology::lexicon::is_delimiter_preposition(normalized) {
            self.has_delimiter_preposition = true;
            return true;
        }

        false
    }

    /// Check for method verbs (split, join)
    fn check_method_verbs(&mut self, normalized: &str) -> bool {
        // Check for split verb
        if crate::morphology::lexicon::is_split_verb(normalized) {
            // If we have a delimiter, create a split method
            #[allow(clippy::collapsible_if)]
            if self.has_delimiter_preposition
                && matches!(
                    self.expression.pending_literals.last(),
                    Some(Literal::String(_))
                )
            {
                if let Some(ref subj) = self.sentence.pending_subject {
                    // Safe to unwrap here because of the checks above
                    let delim = match self.expression.pending_literals.pop() {
                        Some(Literal::String(s)) => s,
                        _ => unreachable!(),
                    };

                    let normalized_original = normalize_greek(&subj.original);
                    self.pending_string_method = Some(("split".to_string(), delim));
                    // Push back a property access for the split result
                    self.expression
                        .pending_property_accesses
                        .push((normalized_original, "split".to_string()));
                }
            }
            return true;
        }

        // Check for join verb
        if crate::morphology::lexicon::is_join_verb(normalized) {
            // If we have a delimiter, create a join method
            #[allow(clippy::collapsible_if)]
            if self.has_delimiter_preposition
                && matches!(
                    self.expression.pending_literals.last(),
                    Some(Literal::String(_))
                )
            {
                if let Some(ref subj) = self.sentence.pending_subject {
                    // Safe to unwrap here because of the checks above
                    let delim = match self.expression.pending_literals.pop() {
                        Some(Literal::String(s)) => s,
                        _ => unreachable!(),
                    };

                    let normalized_original = normalize_greek(&subj.original);
                    self.pending_string_method = Some(("join".to_string(), delim));
                    // Push back a property access for the join result
                    self.expression
                        .pending_property_accesses
                        .push((normalized_original, "join".to_string()));
                }
            }
            return true;
        }

        false
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
    use crate::morphology::lexicon::BinaryOp;
    use crate::morphology::{Voice, analyze};

    #[test]
    fn test_operator_detection() {
        // μεῖζον should be detected as > operator
        let mut asm = Assembler::new();

        // Feed comparison adjective
        let meizon = analyze("μειζον");
        asm.feed(&meizon, "μεῖζον").unwrap();

        let stmt = asm.finalize().unwrap();
        assert!(
            !stmt.operators.is_empty(),
            "Expected operator to be captured"
        );
        assert_eq!(stmt.operators[0], BinaryOp::Gt);
    }

    #[test]
    fn test_boolean_or_detection() {
        // ἤ should be detected as || operator
        let mut asm = Assembler::new();

        // Feed boolean particle
        let or_particle = analyze("η");
        asm.feed(&or_particle, "ἤ").unwrap();

        let stmt = asm.finalize().unwrap();
        assert!(
            !stmt.operators.is_empty(),
            "Expected operator to be captured, got: {:?}",
            stmt
        );
        assert_eq!(stmt.operators[0], BinaryOp::Or);
    }

    #[test]
    fn test_full_boolean_or_expression() {
        // ἀληθές ἤ ψεῦδος λέγε - simulate the full expression
        let mut asm = Assembler::new();

        // Feed true (boolean literal - handled by parser, goes to feed_boolean)
        asm.feed_boolean(true);

        // Feed ἤ (OR operator)
        let or_particle = analyze("η");
        asm.feed(&or_particle, "ἤ").unwrap();

        // Feed false (boolean literal)
        asm.feed_boolean(false);

        // Feed λέγε (print verb)
        let verb = analyze("λεγε");
        asm.feed(&verb, "λέγε").unwrap();

        let stmt = asm.finalize().unwrap();
        assert_eq!(
            stmt.literals.len(),
            2,
            "Expected 2 literals, got: {:?}",
            stmt.literals
        );
        assert_eq!(
            stmt.operators.len(),
            1,
            "Expected 1 operator, got: {:?}",
            stmt.operators
        );
        assert_eq!(stmt.operators[0], BinaryOp::Or);
    }

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
    fn test_multiple_nominatives() {
        // Multiple nominatives are now allowed for function call patterns
        let mut asm = Assembler::new();

        let subj1 = analyze("ανθρωπος");
        asm.feed(&subj1, "ἄνθρωπος").unwrap();

        let subj2 = analyze("θεος");
        asm.feed(&subj2, "θεός").unwrap(); // Should succeed now

        let stmt = asm.finalize().unwrap();
        assert!(stmt.subject.is_some());
        assert_eq!(stmt.nominatives.len(), 1); // Second nominative in the list
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

    #[test]
    fn test_verb_constituent_has_voice() {
        let mut asm = Assembler::new();

        // γίγνεται - middle voice verb
        let verb = analyze("γιγνεται");
        asm.feed(&verb, "γίγνεται").unwrap();

        let stmt = asm.finalize().unwrap();
        assert!(stmt.verb.is_some());
        let verb_const = stmt.verb.unwrap();
        assert_eq!(verb_const.voice, Some(Voice::Middle));
    }
}
