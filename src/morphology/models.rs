//! Core morphology types and models
//!
//! This module contains the foundational types for morphological analysis,
//! extracted to break circular dependencies between `morphology` submodules.

use std::borrow::Cow;

/// Result of morphological analysis
#[derive(Debug, Clone, PartialEq)]
pub struct MorphAnalysis {
    /// The dictionary form (lemma)
    ///
    /// Optimization: Uses `Cow<'static, str>` to avoid allocations for lexicon entries
    /// which are `&'static str`. Only dynamically generated lemmas (e.g. from
    /// suffix stripping) use heap allocation.
    pub lemma: Cow<'static, str>,
    pub part_of_speech: PartOfSpeech,
    pub case: Option<Case>,
    pub number: Option<Number>,
    pub gender: Option<Gender>,
    pub person: Option<Person>,
    pub tense: Option<Tense>,
    pub mood: Option<Mood>,
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
    Noun,
    Verb,
    Adjective,
    Pronoun,
    Article,
    Particle,
    Numeral,
    Preposition,
    Conjunction,
    Adverb,
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
    Nominative,
    Genitive,
    Dative,
    Accusative,
    Vocative,
}

/// Grammatical number
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Number {
    Singular,
    Plural,
    // Dual omitted for MVP
}

/// Grammatical gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

/// Person (for verbs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Person {
    First,
    Second,
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
    Present,
    Imperfect,
    Future,
    Aorist,
    Perfect,
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
    Indicative,
    Imperative,
    Subjunctive,
    Optative,
    Infinitive,
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
    Active,
    Middle,
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
/// use glossa::morphology::{analyses_compatible, MorphAnalysis, PartOfSpeech, Case};
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
