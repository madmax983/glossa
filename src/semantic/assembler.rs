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
//!
//! ## The Hero's Journey: A Sentence's Path
//!
//! Consider the sentence: `ὁ ἄνθρωπος τὸν λόγον λέγει` (The man says the word).
//!
//! 1. **Parsing**: The raw text is tokenized and parsed into an AST.
//! 2. **Analysis**: Each word is morphologically analyzed:
//!    - `ἄνθρωπος`: Noun, Nominative, Singular, Masculine
//!    - `λόγον`: Noun, Accusative, Singular, Masculine
//!    - `λέγει`: Verb, Present, Indicative, Active, 3rd Person, Singular
//! 3. **Assembly**: The `Assembler` receives these analyses:
//!    - `feed("ἄνθρωπος")` -> Sees Nominative -> Places in **Subject** slot.
//!    - `feed("λόγον")` -> Sees Accusative -> Places in **Object** slot.
//!    - `feed("λέγει")` -> Sees Verb -> Places in **Verb** slot.
//! 4. **Finalization**: `finalize()` is called. It checks:
//!    - Does the Subject (Singular) agree with the Verb (Singular)? **Yes.**
//!    - Are there any conflicts? **No.**
//! 5. **Result**: An `AssembledStatement` is born, ready for the next phase.

use crate::ast::{Expr, Word};
pub use crate::errors::AssemblyError;
use crate::morphology::lexicon::BinaryOp;
use crate::morphology::{
    Case, Gender, Mood, MorphAnalysis, Number, PartOfSpeech, Person, Tense, Voice,
};
use crate::text::normalize_greek;
use smol_str::SmolStr;
use unicode_normalization::UnicodeNormalization;

// ==================================================================================
// Data Models (formerly assembly_model.rs)
// ==================================================================================

/// A fully assembled statement with all grammatical roles filled
///
/// This struct represents the "final state" of a sentence after parsing.
/// It contains all the semantic components (subject, verb, object, etc.)
/// extracted from the input stream.
#[derive(Debug, Clone, Default)]
pub struct AssembledStatement {
    /// The subject (nominative) - the agent/doer
    pub subject: Option<Constituent>,

    /// Additional nominatives (for function names, etc.)
    pub nominatives: Vec<Constituent>,

    /// The verb - the action
    pub verb: Option<VerbConstituent>,

    /// The direct object (accusative) - receives the action
    pub object: Option<Constituent>,

    /// The indirect object (dative) - recipient/beneficiary
    pub indirect: Option<Constituent>,

    /// Possessors/sources (genitive) - attached to other constituents
    pub genitives: Vec<Constituent>,

    /// Adjectives modifying nouns
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

    /// Whether this statement propagates (ends with ;)
    pub is_propagate: bool,

    /// Whether this binding has the mutable marker (μετά)
    pub has_mutable_marker: bool,

    /// Whether this statement has the containment preposition (ἐν)
    pub has_containment_preposition: bool,

    /// Whether this statement has the delimiter preposition (κατά)
    pub has_delimiter_preposition: bool,

    /// String method call: (method_name, delimiter)
    pub string_method: Option<(String, String)>,
}

/// A noun/pronoun constituent with its grammatical info
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Constituent {
    /// The dictionary form
    pub lemma: SmolStr,

    /// Original text as it appeared
    pub original: SmolStr,

    /// Normalized text (lowercase, no diacritics)
    pub normalized: SmolStr,

    /// Grammatical case
    pub case: Case,

    /// Grammatical number
    pub number: Option<Number>,

    /// Grammatical gender
    pub gender: Option<Gender>,

    /// Grammatical person (1st, 2nd, 3rd)
    pub person: Option<Person>,
}

/// A verb constituent with its grammatical info
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VerbConstituent {
    /// The dictionary form
    pub lemma: SmolStr,

    /// Original text as it appeared
    pub original: SmolStr,

    /// Normalized text (lowercase, no diacritics)
    pub normalized: SmolStr,

    /// Person (1st, 2nd, 3rd)
    pub person: Option<Person>,

    /// Number (singular, plural)
    pub number: Option<Number>,

    /// Tense (present, aorist, etc.)
    pub tense: Option<Tense>,

    /// Mood (indicative, imperative, etc.)
    pub mood: Option<Mood>,

    /// Voice (active, middle, passive)
    pub voice: Option<Voice>,
}

/// A participle constituent (used for lambdas/closures)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParticipleConstituent {
    /// The verb stem extracted from the participle
    pub verb_lemma: SmolStr,
    /// Original text as it appeared
    pub original: SmolStr,
    /// Normalized text (lowercase, no diacritics)
    pub normalized: SmolStr,
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

// ==================================================================================
// Assembler Logic
// ==================================================================================

// Constants for resource limits to prevent DoS
pub(crate) const MAX_ADJECTIVES: usize = 1024;
pub(crate) const MAX_LITERALS: usize = 1024;
pub(crate) const MAX_NOMINATIVES: usize = 256;
pub(crate) const MAX_GENITIVES: usize = 256;
pub(crate) const MAX_ARRAYS: usize = 256;
pub(crate) const MAX_INDEX_ACCESSES: usize = 256;
pub(crate) const MAX_PROPERTY_ACCESSES: usize = 256;
pub(crate) const MAX_NESTED_PHRASES: usize = 256;
pub(crate) const MAX_PARTICIPLES: usize = 256;
pub(crate) const MAX_UNWRAPS: usize = 256;
pub(crate) const MAX_OPERATORS: usize = 256;
pub(crate) const MAX_BLOCKS: usize = 256;

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
pub(crate) struct Assembler {
    state: AssembledStatement,
}

impl Assembler {
    /// Create a new empty assembler
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
    #[allow(dead_code)]
    pub fn feed(&mut self, analysis: &MorphAnalysis, original: &str) -> Result<(), AssemblyError> {
        let normalized = normalize_greek(original);
        self.feed_with_normalized(analysis, original, &normalized)
    }

    /// Feed a morphologically-analyzed token with pre-computed normalization
    pub fn feed_with_normalized(
        &mut self,
        analysis: &MorphAnalysis,
        original: &str,
        normalized: &str,
    ) -> Result<(), AssemblyError> {
        if self.check_special_markers(normalized, original) {
            return Ok(());
        }

        if self.check_method_verbs(normalized)? {
            return Ok(());
        }

        if self.check_operators(normalized, original)? {
            return Ok(());
        }

        if self.check_special_properties(normalized)? {
            return Ok(());
        }

        match analysis.part_of_speech {
            PartOfSpeech::Noun | PartOfSpeech::Pronoun => {
                self.handle_nominal(analysis, original, normalized)
            }
            PartOfSpeech::Adjective => self.handle_adjective(analysis, original, normalized),
            PartOfSpeech::Verb => self.handle_verb(analysis, original, normalized),
            PartOfSpeech::Numeral => {
                // Already handled above, but keep this for explicit numeral POS
                self.handle_nominal(analysis, original, normalized)
            }
            PartOfSpeech::Conjunction => {
                // Non-operator conjunctions are ignored for now
                Ok(())
            }
            _ => Ok(()), // Ignore particles, articles for now
        }
    }

    /// Feed a string literal
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
            verb_lemma: normalize_greek(&analysis.verb_lemma()),
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

    /// Handle a noun/pronoun/adjective - route to slot by case
    fn handle_nominal(
        &mut self,
        analysis: &MorphAnalysis,
        original: &str,
        normalized: &str,
    ) -> Result<(), AssemblyError> {
        let constituent = Constituent {
            lemma: normalize_greek(analysis.lemma.as_ref()),
            original: original.into(),
            normalized: normalized.into(),
            case: analysis.case.unwrap_or(Case::Nominative),
            number: analysis.number,
            gender: analysis.gender,
            person: analysis.person,
        };

        match analysis.case {
            Some(Case::Nominative) => {
                // If we already have a verb, check agreement immediately!
                if let Some(verb) = &self.state.verb {
                    // Don't check agreement if we already have a subject (this is an extra nominative)
                    if self.state.subject.is_none() {
                        self.check_agreement(&constituent, verb)?;
                    }
                }

                if self.state.subject.is_some() {
                    // Additional nominatives stored separately for function call patterns
                    if self.state.nominatives.len() >= MAX_NOMINATIVES {
                        return Err(AssemblyError::LimitExceeded {
                            resource: "Nominatives".to_string(),
                            max: MAX_NOMINATIVES,
                        });
                    }
                    self.state.nominatives.push(constituent);
                } else {
                    self.state.subject = Some(constituent);
                }
            }
            Some(Case::Accusative) => {
                if self.state.object.is_some() {
                    return Err(AssemblyError::DoubleObject);
                }
                self.state.object = Some(constituent);
            }
            Some(Case::Dative) => {
                // Dative can stack (multiple recipients) but for simplicity, one for now
                if self.state.indirect.is_some() {
                    return Err(AssemblyError::DoubleIndirect);
                }
                self.state.indirect = Some(constituent);
            }
            Some(Case::Genitive) => {
                // Genitives attach to other constituents (possession, etc.)
                if self.state.genitives.len() >= MAX_GENITIVES {
                    return Err(AssemblyError::LimitExceeded {
                        resource: "Genitives".to_string(),
                        max: MAX_GENITIVES,
                    });
                }
                self.state.genitives.push(constituent);
            }
            Some(Case::Vocative) => {
                // Vocative is direct address - treat as subject for now
                if self.state.subject.is_none() {
                    self.state.subject = Some(constituent);
                }
            }
            None => {
                // Unknown case - try to infer from context
                // Default to accusative (object) if we have no object
                if self.state.object.is_none() {
                    self.state.object = Some(constituent);
                } else {
                    return Err(AssemblyError::DoubleObject);
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
        normalized: &str,
    ) -> Result<(), AssemblyError> {
        if self.state.verb.is_some() {
            return Err(AssemblyError::DoubleVerb);
        }

        let verb_constituent = VerbConstituent {
            lemma: normalize_greek(analysis.lemma.as_ref()),
            original: original.into(),
            normalized: normalized.into(),
            person: analysis.person,
            number: analysis.number,
            tense: analysis.tense,
            mood: analysis.mood,
            voice: analysis.voice,
        };

        // If we already have a subject, check agreement immediately!
        if let Some(subject) = &self.state.subject {
            self.check_agreement(subject, &verb_constituent)?;
        }

        self.state.verb = Some(verb_constituent);

        Ok(())
    }

    /// Handle an adjective - store it for later pattern matching
    fn handle_adjective(
        &mut self,
        analysis: &MorphAnalysis,
        original: &str,
        normalized: &str,
    ) -> Result<(), AssemblyError> {
        let constituent = Constituent {
            lemma: normalize_greek(analysis.lemma.as_ref()),
            original: original.into(),
            normalized: normalized.into(),
            case: analysis.case.unwrap_or(Case::Nominative),
            number: analysis.number,
            gender: analysis.gender,
            person: None, // Adjectives don't really have person
        };

        if self.state.adjectives.len() >= MAX_ADJECTIVES {
            return Err(AssemblyError::LimitExceeded {
                resource: "Adjectives".to_string(),
                max: MAX_ADJECTIVES,
            });
        }
        self.state.adjectives.push(constituent);
        Ok(())
    }

    /// Finalize the statement - check agreement and assemble
    ///
    /// This validates the sentence structure (e.g. subject-verb agreement) and
    /// returns the complete `AssembledStatement`.
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
            self.check_agreement(subject, verb)?;
        }

        // Return the assembled statement
        let statement = std::mem::take(&mut self.state);
        Ok(statement)
    }

    /// Check for special markers (mutable, containment, delimiter)
    fn check_special_markers(&mut self, normalized: &str, original: &str) -> bool {
        // Check for mutable marker (μετά)
        if crate::morphology::lexicon::is_mutable_marker(normalized) {
            self.state.has_mutable_marker = true;
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

            self.state.has_containment_preposition = true;
            return true;
        }

        // Check for delimiter preposition (κατά)
        if crate::morphology::lexicon::is_delimiter_preposition(normalized) {
            self.state.has_delimiter_preposition = true;
            return true;
        }

        false
    }

    /// Helper to create a string method call if conditions are met
    #[allow(clippy::collapsible_if)]
    fn try_create_string_method(&mut self, method_name: &str) -> Result<bool, AssemblyError> {
        if self.state.has_delimiter_preposition
            && matches!(self.state.literals.last(), Some(Literal::String(_)))
        {
            if let Some(ref subj) = self.state.subject {
                if self.state.property_accesses.len() >= MAX_PROPERTY_ACCESSES {
                    return Err(AssemblyError::LimitExceeded {
                        resource: "Property Accesses".to_string(),
                        max: MAX_PROPERTY_ACCESSES,
                    });
                }

                let delim = match self.state.literals.pop() {
                    Some(Literal::String(s)) => s,
                    _ => unreachable!(),
                };

                let normalized_original = normalize_greek(&subj.original);
                self.state.string_method = Some((method_name.to_string(), delim));
                self.state
                    .property_accesses
                    .push((normalized_original.to_string(), method_name.to_string()));
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Check for method verbs (split, join)
    fn check_method_verbs(&mut self, normalized: &str) -> Result<bool, AssemblyError> {
        // Check for split verb
        if crate::morphology::lexicon::is_split_verb(normalized)
            && self.try_create_string_method("split")?
        {
            return Ok(true);
        }

        // Check for join verb
        if crate::morphology::lexicon::is_join_verb(normalized)
            && self.try_create_string_method("join")?
        {
            return Ok(true);
        }

        Ok(false)
    }

    /// Check for operators (boolean, comparison, arithmetic)
    fn check_operators(&mut self, normalized: &str, original: &str) -> Result<bool, AssemblyError> {
        // Boolean operators
        if matches!(original, "καί" | "και") {
            if self.state.operators.len() >= MAX_OPERATORS {
                return Err(AssemblyError::LimitExceeded {
                    resource: "Operators".to_string(),
                    max: MAX_OPERATORS,
                });
            }
            self.state.operators.push(BinaryOp::And);
            return Ok(true);
        }
        if matches!(original, "ἤ" | "ή") {
            if self.state.operators.len() >= MAX_OPERATORS {
                return Err(AssemblyError::LimitExceeded {
                    resource: "Operators".to_string(),
                    max: MAX_OPERATORS,
                });
            }
            // ἤ with breathing+accent, but not ᾖ
            self.state.operators.push(BinaryOp::Or);
            return Ok(true);
        }

        // Comparison operators
        if let Some(op) = crate::morphology::lexicon::comparison_operator(normalized) {
            if self.state.operators.len() >= MAX_OPERATORS {
                return Err(AssemblyError::LimitExceeded {
                    resource: "Operators".to_string(),
                    max: MAX_OPERATORS,
                });
            }
            self.state.operators.push(op);
            return Ok(true);
        }

        // Arithmetic operators
        if let Some(op) = crate::morphology::lexicon::arithmetic_operator(normalized) {
            if self.state.operators.len() >= MAX_OPERATORS {
                return Err(AssemblyError::LimitExceeded {
                    resource: "Operators".to_string(),
                    max: MAX_OPERATORS,
                });
            }
            self.state.operators.push(op);
            return Ok(true);
        }

        Ok(false)
    }

    /// Check for special properties (numerals, length, ordinals)
    fn check_special_properties(&mut self, normalized: &str) -> Result<bool, AssemblyError> {
        // Numeral words
        if let Some(value) = crate::morphology::lexicon::numeral_value(normalized) {
            self.state.literals.push(Literal::Number(value));
            return Ok(true);
        }

        // Property nouns (μῆκος)
        if crate::morphology::lexicon::is_length_property(normalized) {
            // If we have a subject, create a property access (use normalized original, not lemma)
            if let Some(ref subj) = self.state.subject {
                if self.state.property_accesses.len() >= MAX_PROPERTY_ACCESSES {
                    return Err(AssemblyError::LimitExceeded {
                        resource: "Property Accesses".to_string(),
                        max: MAX_PROPERTY_ACCESSES,
                    });
                }
                let normalized_original = crate::text::normalize_greek(&subj.original);
                self.state
                    .property_accesses
                    .push((normalized_original.to_string(), "len".to_string()));
                self.state.subject = None; // Consume the subject
                return Ok(true);
            }
        }

        // Ordinal adjectives
        if crate::morphology::lexicon::is_ordinal(normalized) {
            // If we have a subject, create an index access with the ordinal index
            if let Some(ref subj) = self.state.subject
                && let Some(index) = crate::morphology::lexicon::ordinal_to_index(normalized)
            {
                if self.state.index_accesses.len() >= MAX_INDEX_ACCESSES {
                    return Err(AssemblyError::LimitExceeded {
                        resource: "Index Accesses".to_string(),
                        max: MAX_INDEX_ACCESSES,
                    });
                }
                // Create array and index expressions (use normalized original, not lemma)
                let normalized_original = crate::text::normalize_greek(&subj.original);
                let array = Expr::Word(Word {
                    original: subj.original.clone(),
                    normalized: normalized_original.clone(),
                });
                let index_expr = Expr::NumberLiteral(index);

                self.state.index_accesses.push((array, index_expr));
                self.state.subject = None; // Consume the subject
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check subject-verb agreement
    fn check_agreement(
        &self,
        subject: &Constituent,
        verb: &VerbConstituent,
    ) -> Result<(), AssemblyError> {
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
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphology::{Tense, Voice, analyze};
    use std::borrow::Cow;

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
        asm.feed_boolean(true).unwrap();

        // Feed ἤ (OR operator)
        let or_particle = analyze("η");
        asm.feed(&or_particle, "ἤ").unwrap();

        // Feed false (boolean literal)
        asm.feed_boolean(false).unwrap();

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

        asm.feed_string("χαῖρε κόσμε".to_string()).unwrap();

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

    #[test]
    fn test_subject_verb_person_agreement() {
        let mut asm = Assembler::new();

        // Feed subject "ἐγώ" (I) - First Person Singular
        // Manually construct analysis since "ego" might not be in the simple lexicon used in tests
        let ego_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("εγω"),
            part_of_speech: PartOfSpeech::Pronoun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::First), // KEY: First Person
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&ego_analysis, "ἐγώ").unwrap();

        // Feed verb "λέγει" (He says) - Third Person Singular
        let verb_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("λεγω"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::Third), // KEY: Third Person
            tense: Some(Tense::Present),
            mood: Some(Mood::Indicative),
            voice: Some(Voice::Active),
            confidence: 1.0,
        };

        // This should fail IMMEDIATELY during feed because we have strict agreement checks now
        let result = asm.feed(&verb_analysis, "λέγει");

        assert!(
            matches!(result, Err(AssemblyError::SubjectVerbDisagreement { .. })),
            "Expected immediate agreement failure"
        );
    }

    #[test]
    fn test_double_object_error() {
        let mut asm = Assembler::new();

        // First object: λόγον
        let obj1 = analyze("λόγον");
        asm.feed(&obj1, "λόγον").unwrap();

        // Second object: λόγον (again)
        let obj2 = analyze("λόγον");
        let result = asm.feed(&obj2, "λόγον");

        assert!(matches!(result, Err(AssemblyError::DoubleObject)));
    }

    #[test]
    fn test_neuter_plural_subject_singular_verb() {
        let mut asm = Assembler::new();

        // Subject: τὰ ζῷα (The animals) - Neuter Plural
        let subj = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("ζωον"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Plural),
            gender: Some(Gender::Neuter),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&subj, "ζῷα").unwrap();

        // Verb: τρέχει (runs) - Singular
        let verb = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("τρεχω"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular), // Singular!
            gender: None,
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&verb, "τρέχει").unwrap();

        // Should succeed despite Plural Subject + Singular Verb
        let stmt = asm.finalize();
        assert!(
            stmt.is_ok(),
            "Neuter plural subject should agree with singular verb, got {:?}",
            stmt.err()
        );
    }

    #[test]
    fn test_imperative_mismatch() {
        let mut asm = Assembler::new();

        // Subject: "User" (3rd person)
        let subj = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("User"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Masculine),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&subj, "User").unwrap();

        // Verb: "Print!" (Imperative, 2nd person)
        let verb = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("print"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::Second), // 2nd person
            tense: None,
            mood: Some(Mood::Imperative), // Imperative
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&verb, "Print").unwrap();

        // Should succeed
        let stmt = asm.finalize();
        assert!(
            stmt.is_ok(),
            "Imperative verb should allow person mismatch, got {:?}",
            stmt.err()
        );
    }

    #[test]
    fn test_gender_mismatch_ignored() {
        // This test verifies that Gender Mismatch is CURRENTLY IGNORED.
        let mut asm = Assembler::new();

        // Adjective: καλός (Masculine)
        let adj = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("καλος"),
            part_of_speech: PartOfSpeech::Adjective,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Masculine),
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&adj, "καλός").unwrap();

        // Noun: γυνή (Feminine)
        let noun = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("γυνη"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Feminine),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&noun, "γυνή").unwrap();

        // Verb (to complete the sentence)
        let verb = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("λεγω"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&verb, "λέγει").unwrap();

        let stmt = asm.finalize();
        // Currently expecting OK because the check is missing
        assert!(stmt.is_ok(), "Gender mismatch is currently ignored");
    }

    #[test]
    fn test_split_method_generation() {
        let mut asm = Assembler::new();

        // 1. Subject: "text"
        let subj = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("text"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Neuter),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&subj, "text").unwrap();

        // 2. Delimiter Preposition: "κατά"
        let marker_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("κατα"),
            part_of_speech: PartOfSpeech::Preposition,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&marker_analysis, "κατά").unwrap();

        // 3. Delimiter Literal: ","
        asm.feed_string(",".to_string()).unwrap();

        // 4. Split Verb: "σχίζεται" (is split)
        let split_verb = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("σχιζω"), // assuming lemma for split verb
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&split_verb, "σχίζεται").unwrap();

        let stmt = asm.finalize().unwrap();

        // Check if property access was created
        assert!(
            !stmt.property_accesses.is_empty(),
            "Should generate property access for split"
        );
        assert_eq!(stmt.property_accesses[0].1, "split");

        // Check if string method info was captured
        assert_eq!(
            stmt.string_method,
            Some(("split".to_string(), ",".to_string()))
        );
    }

    #[test]
    fn test_immediate_agreement_failure_vso() {
        let mut asm = Assembler::new();

        // Feed verb: "I see" (1st Person Singular)
        let verb_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("βλεπω"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::First),
            tense: Some(Tense::Present),
            mood: Some(Mood::Indicative),
            voice: Some(Voice::Active),
            confidence: 1.0,
        };
        asm.feed(&verb_analysis, "βλέπω").unwrap();

        // Feed subject: "The gift" (3rd Person Singular)
        let subj_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("δωρον"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Neuter),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };

        // Should fail IMMEDIATELY because "I see" (1st) != "gift" (3rd)
        let result = asm.feed(&subj_analysis, "δῶρον");
        assert!(
            matches!(result, Err(AssemblyError::SubjectVerbDisagreement { .. })),
            "Expected immediate agreement failure for VSO"
        );
    }

    #[test]
    fn test_immediate_agreement_failure_svo() {
        let mut asm = Assembler::new();

        // Feed subject: "The gift" (3rd Person Singular)
        let subj_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("δωρον"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Neuter),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&subj_analysis, "δῶρον").unwrap();

        // Feed verb: "I see" (1st Person Singular)
        let verb_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("βλεπω"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::First),
            tense: Some(Tense::Present),
            mood: Some(Mood::Indicative),
            voice: Some(Voice::Active),
            confidence: 1.0,
        };

        // Should fail IMMEDIATELY because "gift" (3rd) != "I see" (1st)
        let result = asm.feed(&verb_analysis, "βλέπω");
        assert!(
            matches!(result, Err(AssemblyError::SubjectVerbDisagreement { .. })),
            "Expected immediate agreement failure for SVO"
        );
    }

    fn make_analysis(
        lemma: &str,
        pos: PartOfSpeech,
        case: Option<Case>,
        number: Option<Number>,
    ) -> MorphAnalysis {
        MorphAnalysis {
            lemma: std::borrow::Cow::Owned(lemma.to_string()),
            part_of_speech: pos,
            case,
            number,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        }
    }

    #[test]
    fn test_max_literals_exceeded() {
        let mut asm = Assembler::new();
        for i in 0..MAX_LITERALS {
            asm.feed_number(i as i64).unwrap();
        }
        let result = asm.feed_number(0);
        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Literals" && max == MAX_LITERALS)
        );
    }

    #[test]
    fn test_max_nominatives_exceeded() {
        let mut asm = Assembler::new();
        let subj = make_analysis(
            "subject",
            PartOfSpeech::Noun,
            Some(Case::Nominative),
            Some(Number::Singular),
        );
        asm.feed(&subj, "subject").unwrap();

        for i in 0..MAX_NOMINATIVES {
            let nom = make_analysis(
                &format!("nom_{}", i),
                PartOfSpeech::Noun,
                Some(Case::Nominative),
                Some(Number::Singular),
            );
            asm.feed(&nom, &format!("nom_{}", i)).unwrap();
        }

        let nom = make_analysis(
            "overflow",
            PartOfSpeech::Noun,
            Some(Case::Nominative),
            Some(Number::Singular),
        );
        let result = asm.feed(&nom, "overflow");

        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Nominatives" && max == MAX_NOMINATIVES)
        );
    }

    #[test]
    fn test_max_adjectives_exceeded() {
        let mut asm = Assembler::new();
        for i in 0..MAX_ADJECTIVES {
            let adj = make_analysis(
                &format!("adj_{}", i),
                PartOfSpeech::Adjective,
                Some(Case::Nominative),
                Some(Number::Singular),
            );
            asm.feed(&adj, &format!("adj_{}", i)).unwrap();
        }

        let adj = make_analysis(
            "overflow",
            PartOfSpeech::Adjective,
            Some(Case::Nominative),
            Some(Number::Singular),
        );
        let result = asm.feed(&adj, "overflow");

        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Adjectives" && max == MAX_ADJECTIVES)
        );
    }

    #[test]
    fn test_max_operators_exceeded() {
        let mut asm = Assembler::new();
        let op_analysis = make_analysis("και", PartOfSpeech::Conjunction, None, None);
        for _ in 0..MAX_OPERATORS {
            asm.feed(&op_analysis, "καί").unwrap();
        }
        let result = asm.feed(&op_analysis, "καί");

        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Operators" && max == MAX_OPERATORS)
        );
    }

    #[test]
    fn test_max_genitives_exceeded() {
        let mut asm = Assembler::new();
        for i in 0..MAX_GENITIVES {
            let genitive = make_analysis(
                &format!("gen_{}", i),
                PartOfSpeech::Noun,
                Some(Case::Genitive),
                Some(Number::Singular),
            );
            asm.feed(&genitive, &format!("gen_{}", i)).unwrap();
        }

        let genitive = make_analysis(
            "overflow",
            PartOfSpeech::Noun,
            Some(Case::Genitive),
            Some(Number::Singular),
        );
        let result = asm.feed(&genitive, "overflow");

        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Genitives" && max == MAX_GENITIVES)
        );
    }

    #[test]
    fn test_max_arrays_exceeded() {
        let mut asm = Assembler::new();
        for _ in 0..MAX_ARRAYS {
            asm.feed_array(vec![]).unwrap();
        }

        match asm.feed_array(vec![]) {
            Err(AssemblyError::LimitExceeded { resource, max }) => {
                assert_eq!(resource, "Arrays");
                assert_eq!(max, MAX_ARRAYS);
            }
            res => panic!("Expected LimitExceeded, got {:?}", res),
        }
    }

    #[test]
    fn test_max_index_accesses_exceeded() {
        let mut asm = Assembler::new();
        let array = Expr::NumberLiteral(1);
        let index = Expr::NumberLiteral(0);

        for _ in 0..MAX_INDEX_ACCESSES {
            asm.feed_index_access(array.clone(), index.clone()).unwrap();
        }

        match asm.feed_index_access(array, index) {
            Err(AssemblyError::LimitExceeded { resource, max }) => {
                assert_eq!(resource, "Index Accesses");
                assert_eq!(max, MAX_INDEX_ACCESSES);
            }
            res => panic!("Expected LimitExceeded, got {:?}", res),
        }
    }

    #[test]
    fn test_max_nested_phrases_exceeded() {
        let mut asm = Assembler::new();
        for _ in 0..MAX_NESTED_PHRASES {
            asm.feed_nested_phrase(vec![]).unwrap();
        }

        match asm.feed_nested_phrase(vec![]) {
            Err(AssemblyError::LimitExceeded { resource, max }) => {
                assert_eq!(resource, "Nested Phrases");
                assert_eq!(max, MAX_NESTED_PHRASES);
            }
            res => panic!("Expected LimitExceeded, got {:?}", res),
        }
    }

    #[test]
    fn test_max_unwraps_exceeded() {
        let mut asm = Assembler::new();
        let expr = Expr::Word(Word::new("x"));

        for _ in 0..MAX_UNWRAPS {
            asm.feed_unwrap(expr.clone()).unwrap();
        }

        match asm.feed_unwrap(expr) {
            Err(AssemblyError::LimitExceeded { resource, max }) => {
                assert_eq!(resource, "Unwraps");
                assert_eq!(max, MAX_UNWRAPS);
            }
            res => panic!("Expected LimitExceeded, got {:?}", res),
        }
    }

    #[test]
    fn test_max_participles_exceeded() {
        let mut asm = Assembler::new();
        let analysis = crate::morphology::ParticipleAnalysis {
            stem: "stem".to_string(),
            tense: crate::morphology::Tense::Present,
            voice: crate::morphology::Voice::Active,
            case: Case::Nominative,
            gender: Gender::Masculine,
            number: Number::Singular,
            confidence: 1.0,
        };

        for _ in 0..MAX_PARTICIPLES {
            asm.feed_participle(&analysis, "part").unwrap();
        }

        match asm.feed_participle(&analysis, "part") {
            Err(AssemblyError::LimitExceeded { resource, max }) => {
                assert_eq!(resource, "Participles");
                assert_eq!(max, MAX_PARTICIPLES);
            }
            res => panic!("Expected LimitExceeded, got {:?}", res),
        }
    }

    #[test]
    fn test_max_blocks_exceeded() {
        let mut asm = Assembler::new();
        for _ in 0..MAX_BLOCKS {
            asm.feed_block(vec![]).unwrap();
        }

        match asm.feed_block(vec![]) {
            Err(AssemblyError::LimitExceeded { resource, max }) => {
                assert_eq!(resource, "Blocks");
                assert_eq!(max, MAX_BLOCKS);
            }
            res => panic!("Expected LimitExceeded, got {:?}", res),
        }
    }

    #[test]
    fn test_limit_operators_or_path() {
        // This tests the check_operators function which returns false on limit
        let mut asm = Assembler::new();
        // Use an analysis that triggers check_operators (e.g. "ἤ")
        let analysis = MorphAnalysis {
            lemma: Cow::Borrowed("η"),
            part_of_speech: PartOfSpeech::Conjunction,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };

        for _ in 0..MAX_OPERATORS {
            asm.feed(&analysis, "ἤ").unwrap();
        }

        match asm.feed(&analysis, "ἤ") {
            Err(AssemblyError::LimitExceeded { resource, max }) => {
                assert_eq!(resource, "Operators");
                assert_eq!(max, MAX_OPERATORS);
            }
            res => panic!("Expected LimitExceeded, got {:?}", res),
        }
    }

    #[test]
    fn test_limit_property_accesses_overflow() {
        // This tests check_special_properties which returns false on limit
        let mut asm = Assembler::new();

        // Setup subject for property access
        let subj = MorphAnalysis {
            lemma: Cow::Borrowed("subj"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Neuter),
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };

        let prop_analysis = MorphAnalysis {
            lemma: Cow::Borrowed("μηκος"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative), // irrelevant
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };

        for _ in 0..MAX_PROPERTY_ACCESSES {
            // Feed subject
            asm.feed(&subj, "text").unwrap();
            // Feed property "μῆκος" (length)
            asm.feed(&prop_analysis, "μῆκος").unwrap();
        }

        // Try one more
        asm.feed(&subj, "text").unwrap();
        match asm.feed(&prop_analysis, "μῆκος") {
            Err(AssemblyError::LimitExceeded { resource, max }) => {
                assert_eq!(resource, "Property Accesses");
                assert_eq!(max, MAX_PROPERTY_ACCESSES);
            }
            res => panic!("Expected LimitExceeded, got {:?}", res),
        }
    }

    #[test]
    fn test_limit_string_method_properties_overflow() {
        // This tests try_create_string_method (called by check_method_verbs)
        let mut asm = Assembler::new();

        let subj = MorphAnalysis {
            lemma: Cow::Borrowed("subj"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Neuter),
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };

        // "κατά" preposition for delimiter
        let delimiter_prep = MorphAnalysis {
            lemma: Cow::Borrowed("κατα"),
            part_of_speech: PartOfSpeech::Preposition,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };

        // "σχίζεται" (is split) verb
        let split_verb = MorphAnalysis {
            lemma: Cow::Borrowed("σχιζω"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };

        // Fill up property accesses first (using standard property access to fill buffer)
        let prop_analysis = MorphAnalysis {
            lemma: Cow::Borrowed("μηκος"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };

        for _ in 0..MAX_PROPERTY_ACCESSES {
            asm.feed(&subj, "text").unwrap();
            asm.feed(&prop_analysis, "μῆκος").unwrap();
        }

        // Now try to trigger a string method which would add another property access
        asm.feed(&subj, "text").unwrap();
        asm.feed(&delimiter_prep, "κατά").unwrap();
        asm.feed_string(",".to_string()).unwrap(); // Delimiter literal

        // Feed split verb - should fail to create method call (return false)
        match asm.feed(&split_verb, "σχίζεται") {
            Err(AssemblyError::LimitExceeded { resource, max }) => {
                assert_eq!(resource, "Property Accesses");
                assert_eq!(max, MAX_PROPERTY_ACCESSES);
            }
            res => panic!("Expected LimitExceeded, got {:?}", res),
        }
    }

    #[test]
    fn test_split_verb_consumes_literal_without_subject_retest() {
        let mut asm = Assembler::new();

        // 1. Feed "κατά" (delimiter preposition)
        let kata = analyze("κατα"); // Should be preposition
        asm.feed(&kata, "κατά").unwrap();

        // 2. Feed string literal " "
        asm.feed_string(" ".to_string()).unwrap();

        // 3. Feed "σχίζεται" (split verb)
        // This triggers check_method_verbs
        let split = analyze("σχιζεται");
        asm.feed(&split, "σχίζεται").unwrap();

        // At this point, if the bug exists:
        // - check_method_verbs returned true (so it was "handled")
        // - pending_literals.pop() was called and consumed " "
        // - pending_subject was None, so no property access was created.

        // 4. Feed "λόγος" (subject)
        let subject = analyze("λογος");
        asm.feed(&subject, "λόγος").unwrap();

        let stmt = asm.finalize().unwrap();

        // If the literal was consumed by the failed split pattern match, it will be missing.
        // If it was preserved, it should be in stmt.literals.
        assert!(
            !stmt.literals.is_empty(),
            "Literal should not be consumed if split pattern fails to match due to missing subject"
        );

        // Also verify that "split" was captured as the verb
        assert!(stmt.verb.is_some(), "Split should be captured as the verb");
        assert_eq!(stmt.verb.unwrap().lemma, "σχιζω");
    }

    #[test]
    fn test_split_verb_not_ignored_without_delimiter_retest() {
        let mut asm = Assembler::new();

        // Feed subject "word"
        let subj = analyze("λογος"); // "word"
        asm.feed(&subj, "λόγος").unwrap();

        // Feed "splits" (σχίζει) without "by" (κατά) and delimiter string
        // normalized: σχιζει
        // This should now be treated as a normal verb because the split pattern didn't match!
        let split_verb = analyze("σχιζει");
        asm.feed(&split_verb, "σχίζει").unwrap();

        let stmt = asm.finalize();

        match stmt {
            Ok(s) => {
                // Now we expect the verb to be present!
                assert!(
                    s.verb.is_some(),
                    "Verb should be present (treated as normal verb) when split pattern fails"
                );
                let verb = s.verb.unwrap();
                assert_eq!(verb.original, "σχίζει");
                assert!(s.string_method.is_none(), "String method should be None");
            }
            Err(e) => {
                panic!("Should not error: {:?}", e);
            }
        }
    }

    #[test]
    fn test_ordinal_not_ignored_without_subject_retest() {
        let mut asm = Assembler::new();

        // Feed "first" (πρῶτον) - Ordinal
        // normalized: πρωτον
        // Since there is no subject yet, it should fall through and be treated as an Adjective
        let first = analyze("πρωτον");
        asm.feed(&first, "πρῶτον").unwrap();

        // Feed "man" (ἄνθρωπος) - Subject
        let man = analyze("ανθρωπος");
        asm.feed(&man, "ἄνθρωπος").unwrap();

        // Feed "is" (ἐστί) - Verb
        let is_verb = analyze("εστι");
        asm.feed(&is_verb, "ἐστί").unwrap();

        let stmt = asm.finalize().unwrap();

        assert!(stmt.subject.is_some(), "Subject should be present");
        assert_eq!(stmt.subject.unwrap().original, "ἄνθρωπος");

        // "first" should be in adjectives now!
        assert!(
            !stmt.adjectives.is_empty(),
            "Adjectives should NOT be empty; 'first' should be captured"
        );
        assert_eq!(stmt.adjectives[0].original, "πρῶτον");

        assert!(
            stmt.index_accesses.is_empty(),
            "Index accesses should be empty"
        );
    }

    #[test]
    fn test_length_property_not_ignored_without_subject_retest() {
        let mut asm = Assembler::new();

        // Feed "length" (μῆκος) - Noun
        // normalized: μηκος
        // Since there is no subject, it should fall through and be treated as a Noun (Subject/Object)
        let len = analyze("μηκος");
        asm.feed(&len, "μῆκος").unwrap();

        // Feed "is" (ἐστί)
        let is_verb = analyze("εστι");
        asm.feed(&is_verb, "ἐστί").unwrap();

        // Feed "5"
        asm.feed_number(5).unwrap();

        let stmt = asm.finalize().unwrap();

        // "length" should be the subject now!
        assert!(stmt.subject.is_some(), "Subject should be present (length)");
        assert_eq!(stmt.subject.unwrap().lemma, "μηκος");

        assert!(
            stmt.property_accesses.is_empty(),
            "Property accesses should be empty"
        );
    }

    #[test]
    fn test_silent_swallowing_of_unknown_case_retest() {
        let mut asm = Assembler::new();
        let obj = make_analysis(
            "object",
            PartOfSpeech::Noun,
            Some(Case::Accusative),
            Some(Number::Singular),
        );
        asm.feed(&obj, "object").unwrap();

        let unknown = make_analysis("unknown", PartOfSpeech::Noun, None, Some(Number::Singular));
        let result = asm.feed(&unknown, "unknown");

        assert!(
            matches!(result, Err(AssemblyError::DoubleObject)),
            "Expected DoubleObject error for unknown case when object slot is full, got {:?}",
            result
        );
    }

    #[test]
    fn test_neuter_plural_subject_first_person_verb_retest() {
        let mut asm = Assembler::new();
        let subj = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("dwron"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Plural),
            gender: Some(Gender::Neuter),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&subj, "δῶρα").unwrap();

        let verb = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("blepw"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::First),
            tense: Some(Tense::Present),
            mood: Some(Mood::Indicative),
            voice: Some(Voice::Active),
            confidence: 1.0,
        };

        let result = asm.feed(&verb, "βλέπω");
        assert!(
            matches!(result, Err(AssemblyError::SubjectVerbDisagreement { .. })),
            "Neuter plural subject (3rd) should NOT agree with 1st person verb, got {:?}",
            result
        );
    }

    #[test]
    fn test_disambiguation_en_vs_hen_retest() {
        let mut asm = Assembler::new();
        let analysis = make_analysis("εν", PartOfSpeech::Preposition, None, None);

        asm.feed(&analysis, "ἐν").unwrap();
        let stmt = asm.finalize().unwrap();
        assert!(stmt.has_containment_preposition);

        let mut asm = Assembler::new();
        let hen = "ἕν";
        asm.feed(&analysis, hen).unwrap();
        let stmt = asm.finalize().unwrap();
        assert!(
            !stmt.has_containment_preposition,
            "Should not detect containment preposition for 'one' (hen)"
        );
    }
}
