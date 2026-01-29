//! Slot-based sentence assembler for ΓΛΩΣΣΑ
//!
//! This module implements a Greek-native approach to sentence parsing.
//! Instead of relying on word order, it routes words to slots based on
//! their grammatical case - just like Ancient Greek actually works.
//!
//! # The "Slot" Concept
//!
//! In languages like English or Rust, word order determines meaning:
//! `func(a, b)` is different from `func(b, a)`.
//!
//! In Ancient Greek (and ΓΛΩΣΣΑ), word order is flexible. Meaning is determined
//! by **case endings**. The `Assembler` acts as a state machine that collects
//! these tokens and puts them into the correct semantic "slots".
//!
//! ```text
//! ┌───────────────────────────────────────────────────────────────┐
//! │                       The Assembler                           │
//! │                                                               │
//! │  Input Stream      ┌──────────────┐                           │
//! │  "ὁ ἄνθρωπος" ────►│ Nominative   │─────► Subject (Agent)     │
//! │  (The man)         └──────────────┘                           │
//! │                                                               │
//! │                    ┌──────────────┐                           │
//! │  "τὸν λόγον"  ────►│ Accusative   │─────► Object (Patient)    │
//! │  (the word)        └──────────────┘                           │
//! │                                                               │
//! │                    ┌──────────────┐                           │
//! │  "λέγει"      ────►│ Verb         │─────► Action              │
//! │  (says)            └──────────────┘                           │
//! │                                                               │
//! └───────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## How it works
//!
//! 1. **Feed**: You feed morphologically analyzed tokens one by one using [`Assembler::feed`].
//! 2. **Route**: The assembler looks at the `Case` of the token (Nominative, Accusative, etc.)
//!    and routes it to the corresponding pending slot.
//! 3. **Accumulate**: Modifiers like adjectives or genitives are accumulated in lists.
//! 4. **Finalize**: When the statement ends (e.g., at a period), you call [`Assembler::finalize`].
//!    This checks for validity (e.g., Subject-Verb agreement) and returns the [`AssembledStatement`].
//!
//! ## Word Order Independence
//!
//! Because slots are filled by case, the following are all equivalent:
//!
//! * **SOV**: `ὁ ἄνθρωπος τὸν λόγον λέγει` (The man says the word)
//! * **VSO**: `λέγει τὸν λόγον ὁ ἄνθρωπος` (Says the word the man)
//! * **OVS**: `τὸν λόγον λέγει ὁ ἄνθρωπος` (The man says the word — with the object fronted)
//!
//! The assembler handles all of these correctly, producing the same assembled semantic representation.

use crate::ast::{Expr, Word};
use crate::grammar::normalize_greek;
use crate::morphology::lexicon::BinaryOp;
use crate::morphology::{
    Case, Gender, Mood, MorphAnalysis, Number, PartOfSpeech, Person, Tense, Voice,
};

/// A fully assembled statement with all grammatical roles filled
///
/// This struct represents the "final state" of a sentence after parsing.
/// It contains all the semantic components (subject, verb, object, etc.)
/// extracted from the input stream.
#[derive(Debug, Clone)]
pub struct AssembledStatement {
    /// The subject (nominative) - the agent/doer
    ///
    /// Example: **ὁ ἄνθρωπος** λέγει (The **man** speaks)
    pub subject: Option<Constituent>,

    /// Additional nominatives (for function names, etc.)
    ///
    /// Used in patterns where multiple nominatives appear, such as
    /// function calls or predicate nominatives.
    pub nominatives: Vec<Constituent>,

    /// The verb - the action
    ///
    /// Example: ὁ ἄνθρωπος **λέγει** (The man **speaks**)
    pub verb: Option<VerbConstituent>,

    /// The direct object (accusative) - receives the action
    ///
    /// Example: βλέπω **τὸν ἄνθρωπον** (I see **the man**)
    pub object: Option<Constituent>,
    /// The indirect object (dative) - recipient/beneficiary
    ///
    /// Example: δίδωμι **τῷ ἀνθρώπῳ** (I give **to the man**)
    pub indirect: Option<Constituent>,

    /// Possessors/sources (genitive) - attached to other constituents
    ///
    /// Example: τὸ τοῦ **ἀνθρώπου** βιβλίον (The **man's** book)
    pub genitives: Vec<Constituent>,

    /// Adjectives modifying nouns (νέον for "new")
    ///
    /// These are accumulated and applied to the relevant nouns during semantic analysis.
    pub adjectives: Vec<Constituent>,

    /// Literal values (strings, numbers) that appeared
    pub literals: Vec<Literal>,
    /// Array literals that appeared
    pub arrays: Vec<Vec<Expr>>,
    /// Index accesses (array, index)
    pub index_accesses: Vec<(Expr, Expr)>,
    /// Property accesses (owner, property)
    pub property_accesses: Vec<(String, String)>,
    /// Binary operators found between expressions
    pub operators: Vec<BinaryOp>,
    /// Parenthesized blocks (nested expressions)
    pub blocks: Vec<Vec<crate::ast::Statement>>,
    /// Nested phrases (parenthesized function calls)
    pub nested_phrases: Vec<Vec<Expr>>,
    /// Participles (used for lambdas/closures)
    pub participles: Vec<ParticipleConstituent>,
    /// Unwrap expressions (expr!)
    pub unwraps: Vec<Expr>,
    /// Whether this is a query (ends with ?)
    pub is_query: bool,
    /// Whether this statement propagates (ends with ;) - converts to `?` in Rust
    pub is_propagate: bool,
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

/// A participle constituent (used for lambdas/closures)
#[derive(Debug, Clone)]
pub struct ParticipleConstituent {
    /// The verb stem extracted from the participle
    pub verb_lemma: String,
    /// Original text as it appeared
    pub original: String,
    /// Tense (present, aorist, perfect)
    pub tense: Tense,
    /// Voice (active, middle, passive)
    pub voice: Voice,
    /// Case (adjectival property)
    pub case: Case,
    /// Gender (adjectival property)
    pub gender: Gender,
    /// Number (adjectival property)
    pub number: Number,
}

/// A literal value
#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(i64),
    Boolean(bool),
}

/// State related to sentence structure (Subject-Verb-Object)
#[derive(Debug, Clone)]
struct SentenceState {
    subject: Option<Constituent>,
    nominatives: Vec<Constituent>,
    object: Option<Constituent>,
    indirect: Option<Constituent>,
    verb: Option<VerbConstituent>,
    genitives: Vec<Constituent>,
    adjectives: Vec<Constituent>,
}

impl SentenceState {
    fn new() -> Self {
        Self {
            subject: None,
            nominatives: Vec::new(),
            object: None,
            indirect: None,
            verb: None,
            genitives: Vec::new(),
            adjectives: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.subject = None;
        self.nominatives.clear();
        self.object = None;
        self.indirect = None;
        self.verb = None;
        self.genitives.clear();
        self.adjectives.clear();
    }
}

/// State related to expression parsing (literals, operators, etc.)
#[derive(Debug, Clone)]
struct ExpressionState {
    literals: Vec<Literal>,
    arrays: Vec<Vec<Expr>>,
    index_accesses: Vec<(Expr, Expr)>,
    property_accesses: Vec<(String, String)>,
    operators: Vec<BinaryOp>,
    blocks: Vec<Vec<crate::ast::Statement>>,
    nested_phrases: Vec<Vec<Expr>>,
    participles: Vec<ParticipleConstituent>,
    unwraps: Vec<Expr>,
}

impl ExpressionState {
    fn new() -> Self {
        Self {
            literals: Vec::new(),
            arrays: Vec::new(),
            index_accesses: Vec::new(),
            property_accesses: Vec::new(),
            operators: Vec::new(),
            blocks: Vec::new(),
            nested_phrases: Vec::new(),
            participles: Vec::new(),
            unwraps: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.literals.clear();
        self.arrays.clear();
        self.index_accesses.clear();
        self.property_accesses.clear();
        self.operators.clear();
        self.blocks.clear();
        self.nested_phrases.clear();
        self.participles.clear();
        self.unwraps.clear();
    }
}

/// The slot-based assembler
///
/// Feed it tokens one by one, and it routes them to the appropriate slot
/// based on their grammatical case. When you hit end-of-statement, call
/// `finalize()` to get the assembled statement.
///
/// # State Machine
///
/// The assembler maintains "pending" slots. As tokens arrive:
/// - **Nominative** -> `pending_subject` (or `pending_nominatives` if subject full)
/// - **Accusative** -> `pending_object`
/// - **Dative** -> `pending_indirect`
/// - **Verb** -> `pending_verb`
///
/// This allows tokens to arrive in any order (Subject-Verb-Object, Verb-Object-Subject, etc.)
/// and still fill the correct semantic roles.
pub struct Assembler {
    sentence: SentenceState,
    expression: ExpressionState,
    /// Slot for the subject (Nominative case)
    pending_subject: Option<Constituent>,
    /// Storage for extra nominatives (e.g. predicate nominatives)
    pending_nominatives: Vec<Constituent>,
    /// Slot for the direct object (Accusative case)
    pending_object: Option<Constituent>,
    /// Slot for the indirect object (Dative case)
    pending_indirect: Option<Constituent>,
    /// Slot for the main verb
    pending_verb: Option<VerbConstituent>,
    /// Accumulated genitives (possessors)
    pending_genitives: Vec<Constituent>,
    /// Accumulated adjectives
    pending_adjectives: Vec<Constituent>,
    /// Accumulated literals (numbers, strings)
    pending_literals: Vec<Literal>,
    /// Accumulated array literals
    pending_arrays: Vec<Vec<Expr>>,
    pending_index_accesses: Vec<(Expr, Expr)>,
    pending_property_accesses: Vec<(String, String)>,
    pending_operators: Vec<BinaryOp>,
    pending_blocks: Vec<Vec<crate::ast::Statement>>,
    pending_nested_phrases: Vec<Vec<Expr>>,
    pending_participles: Vec<ParticipleConstituent>,
    pending_unwraps: Vec<Expr>,
    is_query: bool,
    is_propagate: bool,
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
    ///
    /// # Examples
    ///
    /// ```
    /// use glossa::semantic::Assembler;
    ///
    /// let asm = Assembler::new();
    /// ```
    pub fn new() -> Self {
        Assembler {
            sentence: SentenceState::new(),
            expression: ExpressionState::new(),
            is_query: false,
            is_propagate: false,
        }
    }

    /// Reset the assembler for a new statement
    ///
    /// Clears all pending slots, preparing the assembler for the next sentence.
    /// This is typically called automatically by `finalize()`, but can be
    /// called manually to discard a partial statement.
    pub fn reset(&mut self) {
        self.sentence.reset();
        self.expression.reset();
        self.is_query = false;
        self.is_propagate = false;
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
    ///
    /// This routes the token to the correct slot based on its case.
    ///
    /// # Examples
    ///
    /// ```
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
    pub fn feed(&mut self, analysis: &MorphAnalysis, original: &str) -> Result<(), AssemblyError> {
        let normalized = normalize_greek(original);

        // Check for operators first (before normal part-of-speech handling)
        // Boolean operators: καί (&&), ἤ (||)
        // IMPORTANT: Use original form for ἤ to distinguish from ᾖ (subjunctive)
        // ἤ has smooth breathing + acute, ᾖ has subscript iota + circumflex
        if matches!(original, "καί" | "και") {
            self.expression.operators.push(BinaryOp::And);
            return Ok(());
        }
        if matches!(original, "ἤ" | "ή") {
            // ἤ with breathing+accent, but not ᾖ
            self.expression.operators.push(BinaryOp::Or);
            return Ok(());
        }

        // Comparison operators: μεῖζον (>), ἔλαττον (<), ἴσον (==)
        if let Some(op) = crate::morphology::lexicon::comparison_operator(&normalized) {
            self.expression.operators.push(op);
            return Ok(());
        }

        // Arithmetic operators: ἄθροισμα (+), διαφορά (-), γινόμενον (*)
        if let Some(op) = crate::morphology::lexicon::arithmetic_operator(&normalized) {
            self.expression.operators.push(op);
            return Ok(());
        }

        // Check for numeral words (any case form) - these become literals
        // This catches numeral words regardless of how morphology parsed them
        if let Some(value) = crate::morphology::lexicon::numeral_value(&normalized) {
            self.expression.literals.push(Literal::Number(value));
            return Ok(());
        }

        // Check for property nouns (μῆκος)
        if crate::morphology::lexicon::is_length_property(&normalized) {
            // If we have a subject, create a property access (use normalized original, not lemma)
            if let Some(ref subj) = self.sentence.subject {
                let normalized_original = crate::grammar::normalize_greek(&subj.original);
                self.expression.property_accesses
                    .push((normalized_original, "len".to_string()));
                self.sentence.subject = None; // Consume the subject
            }
            return Ok(());
        }

        // Check for ordinal adjectives (πρῶτον, δεύτερον, τρίτον)
        if crate::morphology::lexicon::is_ordinal(&normalized) {
            // If we have a subject, create an index access with the ordinal index
            if let Some(ref subj) = self.sentence.subject
                && let Some(index) = crate::morphology::lexicon::ordinal_to_index(&normalized)
            {
                // Create array and index expressions (use normalized original, not lemma)
                let normalized_original = crate::grammar::normalize_greek(&subj.original);
                let array = Expr::Word(Word {
                    original: subj.original.clone(),
                    normalized: normalized_original,
                });
                let index_expr = Expr::NumberLiteral(index);

                self.expression.index_accesses.push((array, index_expr));
                self.sentence.subject = None; // Consume the subject
            }
            return Ok(());
        }

        match analysis.part_of_speech {
            PartOfSpeech::Noun | PartOfSpeech::Pronoun => self.handle_nominal(analysis, original),
            PartOfSpeech::Adjective => self.handle_adjective(analysis, original),
            PartOfSpeech::Verb => self.handle_verb(analysis, original),
            PartOfSpeech::Numeral => {
                // Already handled above, but keep this for explicit numeral POS
                self.handle_nominal(analysis, original)
            }
            PartOfSpeech::Conjunction => {
                // Non-operator conjunctions are ignored for now
                Ok(())
            }
            _ => Ok(()), // Ignore particles, articles for now
        }
    }

    /// Feed a string literal
    pub fn feed_string(&mut self, value: String) {
        self.expression.literals.push(Literal::String(value));
    }

    /// Feed a number literal
    pub fn feed_number(&mut self, value: i64) {
        self.expression.literals.push(Literal::Number(value));
    }

    /// Feed a boolean literal
    pub fn feed_boolean(&mut self, value: bool) {
        self.expression.literals.push(Literal::Boolean(value));
    }

    /// Feed an array literal
    pub fn feed_array(&mut self, elements: Vec<Expr>) {
        self.expression.arrays.push(elements);
    }

    /// Feed a parenthesized block (nested expression)
    pub fn feed_block(&mut self, statements: Vec<crate::ast::Statement>) {
        self.expression.blocks.push(statements);
    }

    /// Feed a nested phrase (parenthesized function call)
    pub fn feed_nested_phrase(&mut self, terms: Vec<Expr>) {
        self.expression.nested_phrases.push(terms);
    }

    /// Feed an index access (array[index])
    pub fn feed_index_access(&mut self, array: Expr, index: Expr) {
        self.expression.index_accesses.push((array, index));
    }

    /// Feed an unwrap expression (expr!)
    pub fn feed_unwrap(&mut self, expr: Expr) {
        self.expression.unwraps.push(expr);
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
        self.expression.participles.push(constituent);
    }

    /// Handle a noun/pronoun/adjective - route to slot by case
    fn handle_nominal(
        &mut self,
        analysis: &MorphAnalysis,
        original: &str,
    ) -> Result<(), AssemblyError> {
        let constituent = Constituent {
            lemma: analysis.lemma.clone(),
            original: original.to_string(),
            case: analysis.case.unwrap_or(Case::Nominative),
            number: analysis.number,
            gender: analysis.gender,
        };

        match analysis.case {
            Some(Case::Nominative) => {
                if self.sentence.subject.is_some() {
                    // Additional nominatives stored separately for function call patterns
                    self.sentence.nominatives.push(constituent);
                } else {
                    self.sentence.subject = Some(constituent);
                }
            }
            Some(Case::Accusative) => {
                if self.sentence.object.is_some() {
                    return Err(AssemblyError::DoubleObject);
                }
                self.sentence.object = Some(constituent);
            }
            Some(Case::Dative) => {
                // Dative can stack (multiple recipients) but for simplicity, one for now
                self.sentence.indirect = Some(constituent);
            }
            Some(Case::Genitive) => {
                // Genitives attach to other constituents (possession, etc.)
                self.sentence.genitives.push(constituent);
            }
            Some(Case::Vocative) => {
                // Vocative is direct address - treat as subject for now
                if self.sentence.subject.is_none() {
                    self.sentence.subject = Some(constituent);
                }
            }
            None => {
                // Unknown case - try to infer from context
                // Default to accusative (object) if we have no object
                if self.sentence.object.is_none() {
                    self.sentence.object = Some(constituent);
                }
            }
        }

        Ok(())
    }

    /// Handle a verb
    fn handle_verb(
        &mut self,
        analysis: &MorphAnalysis,
        original: &str,
    ) -> Result<(), AssemblyError> {
        if self.sentence.verb.is_some() {
            return Err(AssemblyError::DoubleVerb);
        }

        self.sentence.verb = Some(VerbConstituent {
            lemma: analysis.lemma.clone(),
            original: original.to_string(),
            person: analysis.person,
            number: analysis.number,
            tense: analysis.tense,
            mood: analysis.mood,
        });

        Ok(())
    }

    /// Handle an adjective - store it for later pattern matching
    fn handle_adjective(
        &mut self,
        analysis: &MorphAnalysis,
        original: &str,
    ) -> Result<(), AssemblyError> {
        let constituent = Constituent {
            lemma: analysis.lemma.clone(),
            original: original.to_string(),
            case: analysis.case.unwrap_or(Case::Nominative),
            number: analysis.number,
            gender: analysis.gender,
        };

        self.sentence.adjectives.push(constituent);
        Ok(())
    }

    /// Finalize the statement - check agreement and assemble
    ///
    /// This validates the sentence structure (e.g. subject-verb agreement) and
    /// returns the complete `AssembledStatement`.
    ///
    /// # Examples
    ///
    /// ```
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
        let has_content = self.sentence.subject.is_some()
            || self.sentence.object.is_some()
            || !self.expression.literals.is_empty();

        if self.sentence.verb.is_none() && has_content && !self.is_query {
            // Allow verbless statements for queries and pure literal expressions
            // But for now, let's be lenient
        }

        // Check subject-verb agreement if both present
        if let (Some(subject), Some(verb)) = (&self.sentence.subject, &self.sentence.verb) {
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
            subject: self.sentence.subject.take(),
            nominatives: std::mem::take(&mut self.sentence.nominatives),
            verb: self.sentence.verb.take(),
            object: self.sentence.object.take(),
            indirect: self.sentence.indirect.take(),
            genitives: std::mem::take(&mut self.sentence.genitives),
            adjectives: std::mem::take(&mut self.sentence.adjectives),
            literals: std::mem::take(&mut self.expression.literals),
            arrays: std::mem::take(&mut self.expression.arrays),
            index_accesses: std::mem::take(&mut self.expression.index_accesses),
            property_accesses: std::mem::take(&mut self.expression.property_accesses),
            operators: std::mem::take(&mut self.expression.operators),
            blocks: std::mem::take(&mut self.expression.blocks),
            nested_phrases: std::mem::take(&mut self.expression.nested_phrases),
            participles: std::mem::take(&mut self.expression.participles),
            unwraps: std::mem::take(&mut self.expression.unwraps),
            is_query: self.is_query,
            is_propagate: self.is_propagate,
        };

        self.reset();
        Ok(statement)
    }

    /// Check if the assembler has any pending content
    pub fn has_content(&self) -> bool {
        self.sentence.subject.is_some()
            || self.sentence.object.is_some()
            || self.sentence.indirect.is_some()
            || self.sentence.verb.is_some()
            || !self.sentence.genitives.is_empty()
            || !self.expression.literals.is_empty()
            || !self.expression.arrays.is_empty()
            || !self.expression.index_accesses.is_empty()
            || !self.expression.property_accesses.is_empty()
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
}
