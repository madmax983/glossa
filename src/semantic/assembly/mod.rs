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
//! # Security & Limits
//!
//! To prevent Denial of Service (DoS) attacks via resource exhaustion (e.g., stack overflow or excessive memory usage),
//! the assembler enforces strict limits on the number of components in a single statement.
//!
//! * **Adjectives**: Max 1024
//! * **Literals**: Max 1024
//! * **Nested Structures**: Max 256 (Arrays, Blocks, Phrases)
//!
//! See [`crate::limits`] for the full list of limits.
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
//!
//! This same process works regardless of the input order.
//!
//! ```rust
//! use glossa::semantic::{Assembler, AssembledStatement};
//! use glossa::morphology::analyze;
//!
//! let mut asm = Assembler::new();
//!
//! // "λέγει" (Verb)
//! asm.feed(&analyze("λέγει"), "λέγει").unwrap();
//!
//! // "τὸν λόγον" (Object)
//! asm.feed(&analyze("λόγον"), "λόγον").unwrap();
//!
//! // "ὁ ἄνθρωπος" (Subject)
//! asm.feed(&analyze("ἄνθρωπος"), "ἄνθρωπος").unwrap();
//!
//! let stmt = asm.finalize().unwrap();
//!
//! assert!(stmt.subject.is_some());
//! assert!(stmt.verb.is_some());
//! assert!(stmt.object.is_some());
//! ```
use crate::ast::{Expr, Word};
pub use crate::errors::AssemblyError;
pub(crate) use crate::limits::{
    MAX_ADJECTIVES, MAX_ARRAYS, MAX_BLOCKS, MAX_GENITIVES, MAX_INDEX_ACCESSES, MAX_LITERALS,
    MAX_NESTED_PHRASES, MAX_NOMINATIVES, MAX_OPERATORS, MAX_PARTICIPLES, MAX_PROPERTY_ACCESSES,
    MAX_UNWRAPS,
};
use crate::morphology::lexicon::BinaryOp;
use crate::morphology::{Case, Gender, Mood, MorphAnalysis, Number, PartOfSpeech, Person};
use crate::text::normalize_greek;
use smol_str::SmolStr;
use unicode_normalization::UnicodeNormalization;

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
struct StatementContext {
    has_only_literals: bool,
    is_operator_expr: bool,
    is_propagate: bool,
    is_string_method: bool,
    is_property_access: bool,
    is_index_access: bool,
    is_nested_phrase: bool,
    is_block: bool,
    is_unwrap: bool,
    is_genitive_possession: bool,
    is_multiple_nominatives: bool,
    is_array: bool,
    has_delimiter: bool,
    is_match_arm: bool,
}
pub(crate) mod model;
pub use model::*;

/// The `Assembler` orchestrates semantic construction.
///
/// It acts as a state machine that collects morphologically analyzed tokens
/// and routes them into the correct semantic "slots" (Subject, Verb, Object)
/// based on their grammatical case, enabling free word order parsing.
///
/// # Examples
///
/// ```rust
/// use glossa::semantic::assembly::Assembler;
/// let mut asm = Assembler::new();
/// // Then feed analysis and finalise statement
/// ```
pub struct Assembler {
    state: AssembledStatement,
}
impl Assembler {
    /// Helper to enforce resource limits within the state.
    fn check_limit(len: usize, max: usize, resource: &str) -> Result<(), AssemblyError> {
        if len >= max {
            Err(AssemblyError::LimitExceeded {
                resource: resource.to_string(),
                max,
            })
        } else {
            Ok(())
        }
    }
    /// Create a new empty assembler
    ///
    /// # Examples
    ///
    /// ```rust
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
    /// ```rust
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
    /// Feeds an AST element into the assembler.
    ///
    /// It parses a string into morphologic traits and saves it to the ongoing statement structure. It exists as the primary interface to collect terms.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// asm.feed(&analysis, "λόγος").unwrap();
    /// ```
    #[allow(dead_code)]
    pub fn feed(&mut self, analysis: &MorphAnalysis, original: &str) -> Result<(), AssemblyError> {
        let normalized = normalize_greek(original);
        self.feed_with_normalized(analysis, original, &normalized)
    }
    /// Feed a morphologically-analyzed token with pre-computed normalization
    ///
    /// Feeds an element into the assembler using its normalized form directly.
    ///
    /// This is a zero-allocation path when the normalized form is already known (e.g. from AST).
    /// It bypasses the costly `normalize_greek` call which may allocate strings.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// asm.feed_with_normalized(&analysis, "λογος", "λόγος").unwrap();
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::semantic::Assembler;
    ///
    /// let mut asm = Assembler::new();
    /// asm.feed_string("χαῖρε".to_string()).unwrap();
    /// ```
    pub fn feed_string(&mut self, value: String) -> Result<(), AssemblyError> {
        Self::check_limit(self.state.literals.len(), MAX_LITERALS, "Literals")?;
        self.state.literals.push(Literal::String(value));
        Ok(())
    }
    /// Feed a number literal
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::semantic::Assembler;
    ///
    /// let mut asm = Assembler::new();
    /// asm.feed_number(42).unwrap();
    /// ```
    pub fn feed_number(&mut self, value: i64) -> Result<(), AssemblyError> {
        Self::check_limit(self.state.literals.len(), MAX_LITERALS, "Literals")?;
        self.state.literals.push(Literal::Number(value));
        Ok(())
    }
    /// Feed a boolean literal
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::semantic::Assembler;
    ///
    /// let mut asm = Assembler::new();
    /// asm.feed_boolean(true).unwrap();
    /// ```
    pub fn feed_boolean(&mut self, value: bool) -> Result<(), AssemblyError> {
        Self::check_limit(self.state.literals.len(), MAX_LITERALS, "Literals")?;
        self.state.literals.push(Literal::Boolean(value));
        Ok(())
    }
    /// Feed an array literal
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::semantic::Assembler;
    /// use glossa::ast::{Expr, Word};
    ///
    /// let mut asm = Assembler::new();
    /// let elements = vec![Expr::NumberLiteral(1), Expr::NumberLiteral(2)];
    /// asm.feed_array(elements).unwrap();
    /// ```
    pub fn feed_array(&mut self, elements: Vec<Expr>) -> Result<(), AssemblyError> {
        Self::check_limit(self.state.arrays.len(), MAX_ARRAYS, "Arrays")?;
        self.state.arrays.push(elements);
        Ok(())
    }
    /// Feed a parenthesized block (nested expression)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::semantic::Assembler;
    ///
    /// let mut asm = Assembler::new();
    /// asm.feed_block(vec![]).unwrap(); // Empty block
    /// ```
    pub fn feed_block(
        &mut self,
        statements: Vec<crate::ast::Statement>,
    ) -> Result<(), AssemblyError> {
        Self::check_limit(self.state.blocks.len(), MAX_BLOCKS, "Blocks")?;
        self.state.blocks.push(statements);
        Ok(())
    }
    /// Feed a nested phrase (parenthesized function call)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::semantic::Assembler;
    /// use glossa::ast::Expr;
    ///
    /// let mut asm = Assembler::new();
    /// asm.feed_nested_phrase(vec![Expr::NumberLiteral(1)]).unwrap();
    /// ```
    pub fn feed_nested_phrase(&mut self, terms: Vec<Expr>) -> Result<(), AssemblyError> {
        Self::check_limit(
            self.state.nested_phrases.len(),
            MAX_NESTED_PHRASES,
            "Nested Phrases",
        )?;
        self.state.nested_phrases.push(terms);
        Ok(())
    }
    /// Feed an index access (`array[index]`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::semantic::Assembler;
    /// use glossa::ast::{Expr, Word};
    /// use smol_str::SmolStr;
    ///
    /// let mut asm = Assembler::new();
    /// let array = Expr::Word(Word {
    ///     original: SmolStr::new("πίναξ"),
    ///     normalized: SmolStr::new("πιναξ"),
    /// });
    /// let index = Expr::NumberLiteral(0);
    /// asm.feed_index_access(array, index).unwrap();
    /// ```
    pub fn feed_index_access(&mut self, array: Expr, index: Expr) -> Result<(), AssemblyError> {
        Self::check_limit(
            self.state.index_accesses.len(),
            MAX_INDEX_ACCESSES,
            "Index Accesses",
        )?;
        self.state.index_accesses.push((array, index));
        Ok(())
    }
    /// Feed an unwrap expression (expr!)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::semantic::Assembler;
    /// use glossa::ast::{Expr, Word};
    /// use smol_str::SmolStr;
    ///
    /// let mut asm = Assembler::new();
    /// let expr = Expr::Word(Word {
    ///     original: SmolStr::new("τιμή"),
    ///     normalized: SmolStr::new("τιμη"),
    /// });
    /// asm.feed_unwrap(expr).unwrap();
    /// ```
    pub fn feed_unwrap(&mut self, expr: Expr) -> Result<(), AssemblyError> {
        Self::check_limit(self.state.unwraps.len(), MAX_UNWRAPS, "Unwraps")?;
        self.state.unwraps.push(expr);
        Ok(())
    }
    /// Feed a participle (for lambda construction)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::semantic::Assembler;
    /// use glossa::morphology::{ParticipleAnalysis, Case, Gender, Number};
    ///
    /// let mut asm = Assembler::new();
    /// let analysis = ParticipleAnalysis {
    ///     stem: "διπλασιαζ".to_string(),
    ///     tense: glossa::morphology::Tense::Present,
    ///     voice: glossa::morphology::Voice::Middle,
    ///     case: Case::Nominative,
    ///     gender: Gender::Neuter,
    ///     number: Number::Plural,
    ///     confidence: 1.0,
    /// };
    ///
    /// asm.feed_participle(&analysis, "διπλασιαζόμενα").unwrap();
    /// ```
    pub fn feed_participle(
        &mut self,
        analysis: &crate::morphology::ParticipleAnalysis,
        original: &str,
    ) -> Result<(), AssemblyError> {
        Self::check_limit(self.state.participles.len(), MAX_PARTICIPLES, "Participles")?;
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
    /// Handle a noun/pronoun/adjective - route to slot by case
    fn handle_nominal(
        &mut self,
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
            Some(Case::Nominative) => self.handle_nominative(constituent),
            Some(Case::Accusative) => self.handle_accusative(constituent),
            Some(Case::Dative) => self.handle_dative(constituent),
            Some(Case::Genitive) => self.handle_genitive(constituent),
            Some(Case::Vocative) => self.handle_vocative(constituent),
            None => self.handle_unknown_case(constituent),
        }
    }
    fn handle_nominative(&mut self, constituent: Constituent) -> Result<(), AssemblyError> {
        // If we already have a verb, check agreement immediately!
        if let Some(verb) = &self.state.verb {
            // Don't check agreement if we already have a subject (this is an extra nominative)
            if self.state.subject.is_none() {
                self.check_agreement(&constituent, verb)?;
            }
        }
        if self.state.subject.is_some() {
            // Additional nominatives stored separately for function call patterns
            Self::check_limit(self.state.nominatives.len(), MAX_NOMINATIVES, "Nominatives")?;
            self.state.nominatives.push(constituent);
        } else {
            self.state.subject = Some(constituent);
        }
        Ok(())
    }
    fn handle_accusative(&mut self, constituent: Constituent) -> Result<(), AssemblyError> {
        if self.state.object.is_some() {
            return Err(AssemblyError::DoubleObject);
        }
        self.state.object = Some(constituent);
        Ok(())
    }
    fn handle_dative(&mut self, constituent: Constituent) -> Result<(), AssemblyError> {
        // Dative can stack (multiple recipients) but for simplicity, one for now
        if self.state.indirect.is_some() {
            return Err(AssemblyError::DoubleIndirect);
        }
        self.state.indirect = Some(constituent);
        Ok(())
    }
    fn handle_genitive(&mut self, constituent: Constituent) -> Result<(), AssemblyError> {
        // Genitives attach to other constituents (possession, etc.)
        Self::check_limit(self.state.genitives.len(), MAX_GENITIVES, "Genitives")?;
        self.state.genitives.push(constituent);
        Ok(())
    }
    fn handle_vocative(&mut self, constituent: Constituent) -> Result<(), AssemblyError> {
        // Vocative is direct address - treat as subject for now
        if self.state.subject.is_none() {
            self.state.subject = Some(constituent);
        }
        Ok(())
    }
    fn handle_unknown_case(&mut self, constituent: Constituent) -> Result<(), AssemblyError> {
        // Unknown case - try to infer from context
        // Default to accusative (object) if we have no object
        if self.state.object.is_none() {
            self.state.object = Some(constituent);
            Ok(())
        } else {
            Err(AssemblyError::DoubleObject)
        }
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
            // OPTIMIZATION: Lemma is guaranteed to be normalized by morphology analysis
            lemma: SmolStr::new(analysis.lemma.as_ref()),
            original: original.into(),
            normalized: normalized.into(),
            case: analysis.case.unwrap_or(Case::Nominative),
            number: analysis.number,
            gender: analysis.gender,
            person: None, // Adjectives don't really have person
        };
        Self::check_limit(self.state.adjectives.len(), MAX_ADJECTIVES, "Adjectives")?;
        self.state.adjectives.push(constituent);
        Ok(())
    }
    /// Finalize the statement - check agreement and assemble
    ///
    /// This validates the sentence structure (e.g. subject-verb agreement) and
    /// returns the complete `AssembledStatement`.
    ///
    /// # Examples
    ///
    /// ```rust
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
            // Exception: pure literal expressions
            let ctx = StatementContext {
                has_only_literals: self.state.subject.is_none() && self.state.object.is_none(),
                is_operator_expr: !self.state.operators.is_empty(),
                is_propagate: self.state.is_propagate,
                is_string_method: self.state.string_method.is_some(),
                is_property_access: !self.state.property_accesses.is_empty(),
                is_index_access: !self.state.index_accesses.is_empty(),
                is_nested_phrase: !self.state.nested_phrases.is_empty(),
                is_block: !self.state.blocks.is_empty(),
                is_unwrap: !self.state.unwraps.is_empty(),
                is_genitive_possession: !self.state.genitives.is_empty(),
                is_multiple_nominatives: !self.state.nominatives.is_empty(),
                is_array: !self.state.arrays.is_empty(),
                has_delimiter: self.state.has_delimiter_preposition,
                is_match_arm: !self.state.adjectives.is_empty()
                    || (self.state.subject.is_some()
                        && self.state.object.is_none()
                        && self.state.literals.is_empty()),
            };
            self.check_missing_verb(&ctx)?;
        }
        // Check subject-verb agreement if both present
        if let (Some(subject), Some(verb)) = (&self.state.subject, &self.state.verb) {
            self.check_agreement(subject, verb)?;
            // If we have a verb, a subject, and extra nominatives, but it's not a function definition or binary operation
            if !self.state.nominatives.is_empty()
                && self.state.operators.is_empty()
                && !crate::morphology::lexicon::is_binding_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_print_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_find_verb(&verb.lemma)
            {
                return Err(AssemblyError::DoubleSubject);
            }
        } else if self.state.subject.is_some()
            && !self.state.nominatives.is_empty()
            && self.state.operators.is_empty()
        {
            // Unhandled double subject when there's no verb and no operators
            // Exception: Function definition / pattern calls
            let is_function_call = !self.state.nested_phrases.is_empty()
                || !self.state.blocks.is_empty()
                || !self.state.literals.is_empty();
            let is_special_pattern =
                !self.state.property_accesses.is_empty() || self.state.is_query;
            // Note: If self.state.verb.is_some(), we would have entered the previous block, not this 'else if' block!
            // Wait, we are in 'else if self.state.subject.is_some()', which means verb is NONE.
            // Oh, so Double Subject logic wasn't fully firing in the previous block either. Let me adjust.
            if !is_function_call && !is_special_pattern {
                // No verb, stacked nominatives...
                return Err(AssemblyError::DoubleSubject);
            }
        }
        // Return the assembled statement
        let statement = std::mem::take(&mut self.state);
        Ok(statement)
    }
    /// Check for missing verb with context
    fn check_missing_verb(&self, ctx: &StatementContext) -> Result<(), AssemblyError> {
        if ctx.has_only_literals
            || ctx.is_operator_expr
            || ctx.is_propagate
            || ctx.is_string_method
            || ctx.is_property_access
            || ctx.is_index_access
            || ctx.is_nested_phrase
            || ctx.is_block
            || ctx.is_unwrap
            || ctx.is_genitive_possession
            || ctx.is_multiple_nominatives
            || ctx.is_array
            || ctx.has_delimiter
        {
            return Ok(());
        }
        if (!self.state.literals.is_empty()
            || !self.state.index_accesses.is_empty()
            || !self.state.property_accesses.is_empty())
            && self.state.subject.is_none()
            && self.state.object.is_none()
        {
            return Ok(());
        }
        if ctx.is_match_arm
            && self.state.object.is_none()
            && self.state.nominatives.is_empty()
            && self.state.adjectives.is_empty()
            && let Some(subject) = self.state.subject.as_ref()
        {
            if subject.lemma == "ανθρωπος" {
                return Err(AssemblyError::MissingVerb);
            }
            return Ok(());
        }
        Err(AssemblyError::MissingVerb)
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
        if self.state.has_delimiter_preposition {
            if let Some(ref subj) = self.state.subject {
                Self::check_limit(
                    self.state.property_accesses.len(),
                    MAX_PROPERTY_ACCESSES,
                    "Property Accesses",
                )?;
                let delim = match self.state.literals.pop() {
                    Some(Literal::String(s)) => s,
                    other => {
                        if let Some(lit) = other {
                            self.state.literals.push(lit);
                        }
                        return Ok(false);
                    }
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
            Self::check_limit(self.state.operators.len(), MAX_OPERATORS, "Operators")?;
            self.state.operators.push(BinaryOp::And);
            return Ok(true);
        }
        if matches!(original, "ἤ" | "ή") {
            Self::check_limit(self.state.operators.len(), MAX_OPERATORS, "Operators")?;
            // ἤ with breathing+accent, but not ᾖ
            self.state.operators.push(BinaryOp::Or);
            return Ok(true);
        }
        // Comparison operators
        if let Some(op) = crate::morphology::lexicon::comparison_operator(normalized) {
            Self::check_limit(self.state.operators.len(), MAX_OPERATORS, "Operators")?;
            self.state.operators.push(op);
            return Ok(true);
        }
        // Arithmetic operators
        if let Some(op) = crate::morphology::lexicon::arithmetic_operator(normalized) {
            Self::check_limit(self.state.operators.len(), MAX_OPERATORS, "Operators")?;
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
                Self::check_limit(
                    self.state.property_accesses.len(),
                    MAX_PROPERTY_ACCESSES,
                    "Property Accesses",
                )?;
                // OPTIMIZATION: Use stored normalized form
                self.state
                    .property_accesses
                    .push((subj.normalized.to_string(), "len".to_string()));
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
                Self::check_limit(
                    self.state.index_accesses.len(),
                    MAX_INDEX_ACCESSES,
                    "Index Accesses",
                )?;
                // Create array and index expressions (use normalized original, not lemma)
                // OPTIMIZATION: Use stored normalized form
                let array = Expr::Word(Word {
                    original: subj.original.clone(),
                    normalized: subj.normalized.clone(),
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
mod tests;
