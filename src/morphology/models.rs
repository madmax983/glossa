//! Core morphology types and models
//!
//! This module contains the foundational types for morphological analysis,
//! extracted to break circular dependencies between `morphology` submodules.

use std::borrow::Cow;

/// Result of morphological analysis
///
/// Every word parsed in ΓΛΩΣΣΑ undergoes a transformation from raw text into a `MorphAnalysis`.
/// It is the philosopher's stone that reveals the true grammatical essence (εἶδος) of a word,
/// mapping its outer form to its internal semantic potential (case, number, tense).
///
/// ## Examples
///
/// ```rust
/// use glossa::morphology::models::{MorphAnalysis, PartOfSpeech, Case, Number};
///
/// // Create a manual analysis for the word "λόγος" (word/reason)
/// let mut analysis = MorphAnalysis::new("λογος".to_string(), PartOfSpeech::Noun);
/// analysis.case = Some(Case::Nominative);
/// analysis.number = Some(Number::Singular);
///
/// assert_eq!(analysis.lemma, "λογος");
/// assert_eq!(analysis.part_of_speech, PartOfSpeech::Noun);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MorphAnalysis {
    /// The dictionary form (lemma)
    ///
    /// Optimization: Uses `Cow<'static, str>` to avoid allocations for lexicon entries
    /// which are `&'static str`. Only dynamically generated lemmas (e.g. from
    /// suffix stripping) use heap allocation.
    pub lemma: Cow<'static, str>,
    /// The fundamental grammatical category of the word
    pub part_of_speech: PartOfSpeech,
    /// The grammatical case, determining semantic role in a sentence
    pub case: Option<Case>,
    /// The grammatical number (singular or plural)
    pub number: Option<Number>,
    /// The grammatical gender (masculine, feminine, neuter)
    pub gender: Option<Gender>,
    /// The grammatical person (first, second, third)
    pub person: Option<Person>,
    /// The grammatical tense (present, future, past etc.)
    pub tense: Option<Tense>,
    /// The grammatical mood (indicative, imperative, etc.)
    pub mood: Option<Mood>,
    /// The grammatical voice (active, middle, passive)
    pub voice: Option<Voice>,
    /// Confidence score (0.0 - 1.0). Higher = more likely.
    /// Lexicon entries get 1.0, pattern matches get lower scores.
    pub confidence: f32,
}

impl MorphAnalysis {
    /// Create a new analysis with default confidence
    pub fn new(lemma: String, pos: PartOfSpeech) -> Self {
        MorphAnalysis {
            lemma: Cow::Owned(lemma),
            part_of_speech: pos,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 0.5,
        }
    }

    /// Set confidence score
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
}

/// Part of speech
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartOfSpeech {
    /// Nouns: words denoting things, concepts, or entities (e.g. types, data structures).
    Noun,
    /// Verbs: words denoting actions or states (e.g. functions, operations).
    Verb,
    /// Adjectives: words describing nouns (e.g. boolean conditions, properties).
    Adjective,
    /// Pronouns: words substituting for nouns (e.g. `τι`, `πάντα`).
    Pronoun,
    /// Articles: words indicating definiteness (e.g. `ὁ`, `ἡ`, `τό`).
    Article,
    /// Particles: small, uninflected words for structure or emphasis (e.g. `εἰ`, `οὐ`).
    Particle,
    /// Numerals: numbers and counting words (e.g. `πέντε`, `χίλια`).
    Numeral,
    /// Prepositions: words indicating relationships like direction or containment (e.g. `ἐν`, `εἰς`).
    Preposition,
    /// Conjunctions: words connecting clauses or words (e.g. `καί`).
    Conjunction,
    /// Adverbs: words modifying verbs, adjectives, or other adverbs.
    Adverb,
    /// Unknown part of speech, usually indicating a parsing failure or missing vocabulary.
    Unknown,
}

/// Grammatical case - determines semantic role
///
/// In ΓΛΩΣΣΑ:
/// - Nominative: the subject/agent
/// - Genitive: possession, property access, borrow (&T)
/// - Dative: indirect object, recipient, mutable borrow (&mut T)
/// - Accusative: direct object, function argument, move/ownership
/// - Vocative: direct address (for error messages)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Case {
    /// The subject or agent of an action (who or what performs the verb).
    Nominative,
    /// Indicates possession, property access, or a reference (`&T`).
    Genitive,
    /// The indirect object, recipient, or a mutable reference (`&mut T`).
    Dative,
    /// The direct object, receiving the action, or representing ownership/move semantics.
    Accusative,
    /// Used for direct address, such as calling out errors or user interactions.
    Vocative,
}

/// Grammatical number
///
/// Defines the magnitude or multiplicity of an entity.
/// In ΓΛΩΣΣΑ, this dictates whether an operation is performed once (singular)
/// or mapped across a collection (plural).
///
/// ## Examples
///
/// ```rust
/// use glossa::morphology::models::Number;
///
/// let one = Number::Singular;
/// let many = Number::Plural;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Number {
    /// Indicates an isolated, indivisible unit (`ἕν`).
    Singular,
    /// Indicates a multitude, collection, or an iterable sequence (`πολλά`).
    Plural,
    // Dual omitted for MVP
}

/// Grammatical gender
///
/// In ancient Greek philosophy, gender (γένος) categorized the natural world.
/// In ΓΛΩΣΣΑ, it serves as a strict compile-time typing mechanism for pronouns
/// and adjectives, ensuring that descriptive properties align perfectly with
/// their target entities.
///
/// ## Examples
///
/// ```rust
/// use glossa::morphology::models::Gender;
///
/// // 'ἀριθμός' (number) takes masculine modifiers
/// let num_gender = Gender::Masculine;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Gender {
    /// The primary active or conceptual grouping, typically applied to abstract structures.
    Masculine,
    /// The receptive or contained grouping, often applied to collections like `λίσται` (lists).
    Feminine,
    /// The objective or literal grouping, predominantly used for raw data types like `ὄνομα` (string).
    Neuter,
}

/// Person (for verbs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Person {
    /// The speaker (I/We)
    First,
    /// The addressee (You)
    Second,
    /// The subject being spoken about (He/She/It/They)
    Third,
}

/// Tense - encodes aspect/time
///
/// In ΓΛΩΣΣΑ:
/// - Present: streaming/iterative operations
/// - Aorist: one-shot/complete operations
/// - Perfect: completed with lasting result
/// - Imperfect: ongoing past (for async?)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Tense {
    /// Present tense, often used for continuous or streaming operations.
    Present,
    /// Imperfect tense, indicating ongoing past actions (potential async).
    Imperfect,
    /// Future tense, for operations to happen later.
    Future,
    /// Aorist tense, indicating a discrete, one-shot, completed operation.
    Aorist,
    /// Perfect tense, for completed operations with lasting current results.
    Perfect,
    /// Pluperfect tense, for operations completed before another past event.
    Pluperfect,
}

/// Mood - encodes modality
///
/// In ΓΛΩΣΣΑ:
/// - Indicative: statements of fact, regular execution
/// - Imperative: commands, top-level expressions
/// - Subjunctive: conditionals, possibility
/// - Optative: wishes, optional execution
/// - Infinitive: non-finite verb form
/// - Participle: verbal adjective, used for lambdas/closures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Mood {
    /// Statements of fact or regular execution paths.
    Indicative,
    /// Commands or top-level execution expressions.
    Imperative,
    /// Expresses possibility, conditionals, or hypothetical execution.
    Subjunctive,
    /// Expresses wishes or optional execution.
    Optative,
    /// The abstract, non-finite form of a verb, often used for definitions.
    Infinitive,
    /// A verbal adjective, effectively acting as lambdas or closures.
    Participle,
}

/// Voice - active, middle, passive
///
/// In ΓΛΩΣΣΑ:
/// - Active: regular function calls
/// - Middle: reflexive/self-affecting (method on self)
/// - Passive: subject receives action (callback/event handler)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Voice {
    /// The subject acts on an object (regular function call).
    Active,
    /// The subject acts upon itself (method call on `self`).
    Middle,
    /// The subject receives the action (callbacks, event handlers).
    Passive,
}

impl std::fmt::Display for Case {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Case::Nominative => write!(f, "ὀνομαστική"),
            Case::Genitive => write!(f, "γενική"),
            Case::Dative => write!(f, "δοτική"),
            Case::Accusative => write!(f, "αἰτιατική"),
            Case::Vocative => write!(f, "κλητική"),
        }
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Singular => write!(f, "ἑνικός"),
            Number::Plural => write!(f, "πληθυντικός"),
        }
    }
}

impl std::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gender::Masculine => write!(f, "ἀρσενικόν"),
            Gender::Feminine => write!(f, "θηλυκόν"),
            Gender::Neuter => write!(f, "οὐδέτερον"),
        }
    }
}

/// Check if two analyses are compatible (could refer to the same word in context)
///
/// Returns `true` if the analyses do not have conflicting grammatical features.
/// Used during disambiguation to check if a word's potential analysis fits
/// with the surrounding context (e.g., article agreement).
///
/// # Examples
///
/// ```
/// use glossa::morphology::models::{analyses_compatible, MorphAnalysis, PartOfSpeech, Case};
///
/// let mut a = MorphAnalysis::new("λογος".to_string(), PartOfSpeech::Noun);
/// a.case = Some(Case::Nominative);
///
/// let mut b = MorphAnalysis::new("λογος".to_string(), PartOfSpeech::Noun);
/// b.case = Some(Case::Accusative);
///
/// assert!(!analyses_compatible(&a, &b));
/// ```
pub fn analyses_compatible(a: &MorphAnalysis, b: &MorphAnalysis) -> bool {
    // Check case agreement if both have cases
    if let (Some(case_a), Some(case_b)) = (a.case, b.case)
        && case_a != case_b
    {
        return false;
    }

    // Check number agreement if both have numbers
    if let (Some(num_a), Some(num_b)) = (a.number, b.number)
        && num_a != num_b
    {
        return false;
    }

    // Check gender agreement if both have genders
    if let (Some(gen_a), Some(gen_b)) = (a.gender, b.gender)
        && gen_a != gen_b
    {
        return false;
    }

    true
}
