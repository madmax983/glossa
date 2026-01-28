//! Morphological categories for Ancient Greek
//!
//! These enums represent the grammatical categories that encode
//! semantic roles in ΓΛΩΣΣΑ.

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

impl Case {
    /// Convert case to its ownership/borrowing semantic
    pub fn to_rust_ownership(&self) -> &'static str {
        match self {
            Case::Nominative => "", // Subject, just the value
            Case::Genitive => "&",  // Borrow (of/from)
            Case::Dative => "&mut", // Mutable borrow (to/for)
            Case::Accusative => "", // Move (direct object)
            Case::Vocative => "",   // No ownership semantic
        }
    }
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

impl Tense {
    /// Convert tense to execution semantics
    pub fn execution_mode(&self) -> ExecutionMode {
        match self {
            Tense::Present => ExecutionMode::Streaming,
            Tense::Aorist => ExecutionMode::OneShot,
            Tense::Perfect => ExecutionMode::Cached,
            Tense::Future => ExecutionMode::Lazy,
            _ => ExecutionMode::OneShot,
        }
    }
}

/// Execution mode derived from verbal aspect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Streaming/iterative - process elements one by one
    Streaming,
    /// One-shot - execute once, consume input
    OneShot,
    /// Cached - compute once, reuse result
    Cached,
    /// Lazy - defer until needed
    Lazy,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_ownership() {
        assert_eq!(Case::Genitive.to_rust_ownership(), "&");
        assert_eq!(Case::Dative.to_rust_ownership(), "&mut");
        assert_eq!(Case::Accusative.to_rust_ownership(), "");
    }

    #[test]
    fn test_tense_execution_mode() {
        assert_eq!(Tense::Present.execution_mode(), ExecutionMode::Streaming);
        assert_eq!(Tense::Aorist.execution_mode(), ExecutionMode::OneShot);
        assert_eq!(Tense::Perfect.execution_mode(), ExecutionMode::Cached);
    }
}
