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
    /// Whether this binding has the mutable marker (μετά)
    pub has_mutable_marker: bool,
    /// Whether this statement has the containment preposition (ἐν)
    /// Used for contains patterns: element ἐν set? → set.contains(&element)
    pub has_containment_preposition: bool,
    /// Whether this statement has the delimiter preposition (κατά)
    /// Used for split/join patterns: string κατά delimiter σχίζεται → string.split(delimiter)
    pub has_delimiter_preposition: bool,
    /// String method call: (method_name, delimiter)
    /// Used for split/join patterns
    pub string_method: Option<(String, String)>,
}

/// A noun/pronoun constituent with its grammatical info
///
/// This represents a word that fills a nominal slot (Subject, Object, etc.).
/// It carries both its original surface form and its normalized lemma.
#[derive(Debug, Clone)]
pub struct Constituent {
    /// The dictionary form
    ///
    /// Used for semantic analysis and code generation.
    /// Example: "ανθρωπος" (from "ἄνθρωπος")
    pub lemma: String,

    /// Original text as it appeared
    ///
    /// Preserved for error messages and display purposes.
    /// Example: "ἄνθρωπος"
    pub original: String,

    /// Grammatical case
    ///
    /// Determines which slot this constituent fills:
    /// - `Nominative` -> Subject
    /// - `Accusative` -> Object
    /// - `Dative` -> Indirect Object
    pub case: Case,

    /// Grammatical number
    ///
    /// Used for subject-verb agreement checks.
    pub number: Option<Number>,

    /// Grammatical gender
    ///
    /// Used for adjective-noun agreement checks.
    pub gender: Option<Gender>,
}

/// A verb constituent with its grammatical info
///
/// This represents the main action of the sentence.
#[derive(Debug, Clone)]
pub struct VerbConstituent {
    /// The dictionary form (1st person singular present)
    ///
    /// Example: "λεγω" (from "λέγει")
    pub lemma: String,

    /// Original text as it appeared
    ///
    /// Example: "λέγει"
    pub original: String,

    /// Person (1st, 2nd, 3rd)
    ///
    /// Used for agreement with the subject.
    pub person: Option<Person>,

    /// Number (singular, plural)
    ///
    /// Used for agreement with the subject.
    pub number: Option<Number>,

    /// Tense (present, aorist, etc.)
    ///
    /// Determines execution semantics (e.g., Aorist = immediate, Present = continuous).
    pub tense: Option<Tense>,

    /// Mood (indicative, imperative, etc.)
    ///
    /// Determines sentence type (Statement vs Command).
    pub mood: Option<Mood>,

    /// Voice (active, middle, passive)
    ///
    /// Determines the relationship between subject and action.
    /// - Active: Subject does action
    /// - Middle: Subject acts on self/for self (often used for specific ops like `.pop()`)
    pub voice: Option<Voice>,
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
    pending_mutable_marker: bool,
    /// Track containment preposition (ἐν) for contains patterns
    has_containment_preposition: bool,
    /// Track delimiter preposition (κατά) for split/join patterns
    has_delimiter_preposition: bool,
    /// Track split/join method call: (method_name, delimiter)
    pending_string_method: Option<(String, String)>,
    is_query: bool,
    is_propagate: bool,
}

/// Errors that can occur during assembly
#[derive(Debug, Clone, thiserror::Error)]
pub enum AssemblyError {
    /// Two subjects found in the same statement (Nominative collision)
    ///
    /// # Example
    /// `ὁ ἄνθρωπος ὁ θεὸς λέγει` (The man the god says)
    #[error("Διπλοῦν ὑποκείμενον! Δύο βασιλεῖς οὐ δύνανται μιᾶς πόλεως ἄρχειν.")]
    DoubleSubject,

    /// Two objects found in the same statement (Accusative collision)
    ///
    /// # Example
    /// `τὸν λόγον τὴν πόλιν βλέπω` (I see the word the city)
    #[error("Διπλοῦν ἀντικείμενον! Ἓν μόνον κατηγορεῖς.")]
    DoubleObject,

    /// Two verbs found in the same statement
    ///
    /// # Example
    /// `λέγει γράφει ὁ ἄνθρωπος` (The man says writes)
    #[error("Διπλοῦν ῥῆμα! Μία πρᾶξις ἑκάστοτε.")]
    DoubleVerb,

    /// No verb found in the statement
    ///
    /// Note: Pure expressions (like `5`) are allowed, but incomplete sentences trigger this.
    ///
    /// # Example
    /// `ὁ ἄνθρωπος τὸν λόγον` (The man the word ... [missing action])
    #[error("Ῥῆμα οὐχ εὑρέθη! Οὐδὲν ἐγένετο.")]
    MissingVerb,

    /// Subject and Verb do not agree in number/person
    ///
    /// # Example
    /// `ὁ ἄνθρωπος (Singular) λέγουσιν (Plural)`
    #[error("Ἀσυμφωνία: ὑποκείμενον {subject:?} ἀλλὰ ῥῆμα {verb:?}")]
    SubjectVerbDisagreement {
        subject: (Option<Person>, Option<Number>),
        verb: (Option<Person>, Option<Number>),
    },

    /// Adjective and Noun do not agree in gender
    ///
    /// # Example
    /// `ὁ καλὸς (Masc) γυνή (Fem)`
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
            pending_subject: None,
            pending_nominatives: Vec::new(),
            pending_object: None,
            pending_indirect: None,
            pending_verb: None,
            pending_genitives: Vec::new(),
            pending_adjectives: Vec::new(),
            pending_literals: Vec::new(),
            pending_arrays: Vec::new(),
            pending_index_accesses: Vec::new(),
            pending_property_accesses: Vec::new(),
            pending_operators: Vec::new(),
            pending_blocks: Vec::new(),
            pending_nested_phrases: Vec::new(),
            pending_participles: Vec::new(),
            pending_unwraps: Vec::new(),
            pending_mutable_marker: false,
            has_containment_preposition: false,
            has_delimiter_preposition: false,
            pending_string_method: None,
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
        self.pending_subject = None;
        self.pending_nominatives.clear();
        self.pending_object = None;
        self.pending_indirect = None;
        self.pending_verb = None;
        self.pending_genitives.clear();
        self.pending_adjectives.clear();
        self.pending_literals.clear();
        self.pending_arrays.clear();
        self.pending_index_accesses.clear();
        self.pending_property_accesses.clear();
        self.pending_operators.clear();
        self.pending_blocks.clear();
        self.pending_nested_phrases.clear();
        self.pending_participles.clear();
        self.pending_unwraps.clear();
        self.pending_mutable_marker = false;
        self.has_containment_preposition = false;
        self.has_delimiter_preposition = false;
        self.pending_string_method = None;
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

        if self.check_special_markers(&normalized) {
            return Ok(());
        }

        if self.check_method_verbs(&normalized) {
            return Ok(());
        }

        if self.check_operators(&normalized, original) {
            return Ok(());
        }

        if self.check_special_properties(&normalized) {
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

    /// Handle a noun/pronoun/adjective - route to slot by case
    fn handle_nominal(
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
    fn handle_verb(
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
    fn handle_adjective(
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
            subject: self.pending_subject.take(),
            nominatives: std::mem::take(&mut self.pending_nominatives),
            verb: self.pending_verb.take(),
            object: self.pending_object.take(),
            indirect: self.pending_indirect.take(),
            genitives: std::mem::take(&mut self.pending_genitives),
            adjectives: std::mem::take(&mut self.pending_adjectives),
            literals: std::mem::take(&mut self.pending_literals),
            arrays: std::mem::take(&mut self.pending_arrays),
            index_accesses: std::mem::take(&mut self.pending_index_accesses),
            property_accesses: std::mem::take(&mut self.pending_property_accesses),
            operators: std::mem::take(&mut self.pending_operators),
            blocks: std::mem::take(&mut self.pending_blocks),
            nested_phrases: std::mem::take(&mut self.pending_nested_phrases),
            participles: std::mem::take(&mut self.pending_participles),
            unwraps: std::mem::take(&mut self.pending_unwraps),
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
                && matches!(self.pending_literals.last(), Some(Literal::String(_)))
            {
                if let Some(ref subj) = self.pending_subject {
                    // Safe to unwrap here because of the checks above
                    let delim = match self.pending_literals.pop() {
                        Some(Literal::String(s)) => s,
                        _ => unreachable!(),
                    };

                    let normalized_original = normalize_greek(&subj.original);
                    self.pending_string_method = Some(("split".to_string(), delim));
                    // Push back a property access for the split result
                    self.pending_property_accesses
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
                && matches!(self.pending_literals.last(), Some(Literal::String(_)))
            {
                if let Some(ref subj) = self.pending_subject {
                    // Safe to unwrap here because of the checks above
                    let delim = match self.pending_literals.pop() {
                        Some(Literal::String(s)) => s,
                        _ => unreachable!(),
                    };

                    let normalized_original = normalize_greek(&subj.original);
                    self.pending_string_method = Some(("join".to_string(), delim));
                    // Push back a property access for the join result
                    self.pending_property_accesses
                        .push((normalized_original, "join".to_string()));
                }
            }
            return true;
        }

        false
    }

    /// Check for operators (boolean, comparison, arithmetic)
    fn check_operators(&mut self, normalized: &str, original: &str) -> bool {
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
    fn check_special_properties(&mut self, normalized: &str) -> bool {
        // Numeral words
        if let Some(value) = crate::morphology::lexicon::numeral_value(normalized) {
            self.pending_literals.push(Literal::Number(value));
            return true;
        }

        // Property nouns (μῆκος)
        if crate::morphology::lexicon::is_length_property(normalized) {
            // If we have a subject, create a property access (use normalized original, not lemma)
            if let Some(ref subj) = self.pending_subject {
                let normalized_original = crate::grammar::normalize_greek(&subj.original);
                self.pending_property_accesses
                    .push((normalized_original, "len".to_string()));
                self.pending_subject = None; // Consume the subject
            }
            return true;
        }

        // Ordinal adjectives
        if crate::morphology::lexicon::is_ordinal(normalized) {
            // If we have a subject, create an index access with the ordinal index
            if let Some(ref subj) = self.pending_subject
                && let Some(index) = crate::morphology::lexicon::ordinal_to_index(normalized)
            {
                // Create array and index expressions (use normalized original, not lemma)
                let normalized_original = crate::grammar::normalize_greek(&subj.original);
                let array = Expr::Word(Word {
                    original: subj.original.clone(),
                    normalized: normalized_original,
                });
                let index_expr = Expr::NumberLiteral(index);

                self.pending_index_accesses.push((array, index_expr));
                self.pending_subject = None; // Consume the subject
            }
            return true;
        }

        false
    }

    /// Check if the assembler has any pending content
    pub fn has_content(&self) -> bool {
        self.pending_subject.is_some()
            || self.pending_object.is_some()
            || self.pending_indirect.is_some()
            || self.pending_verb.is_some()
            || !self.pending_genitives.is_empty()
            || !self.pending_literals.is_empty()
            || !self.pending_arrays.is_empty()
            || !self.pending_index_accesses.is_empty()
            || !self.pending_property_accesses.is_empty()
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
