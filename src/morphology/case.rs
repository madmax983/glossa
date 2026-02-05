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

    /// Get the Greek name for the case
    pub fn to_greek(&self) -> &'static str {
        match self {
            Case::Nominative => "Ὀνομαστική",
            Case::Genitive => "Γενική",
            Case::Dative => "Δοτική",
            Case::Accusative => "Αἰτιατική",
            Case::Vocative => "Κλητική",
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

impl Number {
    /// Get the Greek name for the number
    pub fn to_greek(&self) -> &'static str {
        match self {
            Number::Singular => "Ἑνικός",
            Number::Plural => "Πληθυντικός",
        }
    }
}

/// Grammatical gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

impl Gender {
    /// Get the Greek name for the gender
    pub fn to_greek(&self) -> &'static str {
        match self {
            Gender::Masculine => "Ἀρσενικόν",
            Gender::Feminine => "Θηλυκόν",
            Gender::Neuter => "Οὐδέτερον",
        }
    }
}

/// Person (for verbs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Person {
    First,
    Second,
    Third,
}

impl Person {
    /// Get the Greek name for the person
    pub fn to_greek(&self) -> &'static str {
        match self {
            Person::First => "Πρῶτον",
            Person::Second => "Δεύτερον",
            Person::Third => "Τρίτον",
        }
    }
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
    /// Get the Greek name for the tense
    pub fn to_greek(&self) -> &'static str {
        match self {
            Tense::Present => "Ἐνεστώς",
            Tense::Imperfect => "Παρατατικός",
            Tense::Future => "Μέλλων",
            Tense::Aorist => "Ἀόριστος",
            Tense::Perfect => "Παρακείμενος",
            Tense::Pluperfect => "Ὑπερσυντέλικος",
        }
    }
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

impl Mood {
    /// Get the Greek name for the mood
    pub fn to_greek(&self) -> &'static str {
        match self {
            Mood::Indicative => "Ὁριστική",
            Mood::Imperative => "Προστακτική",
            Mood::Subjunctive => "Ὑποτακτική",
            Mood::Optative => "Εὐκτική",
            Mood::Infinitive => "Ἀπαρέμφατον",
            Mood::Participle => "Μετοχή",
        }
    }
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

impl Voice {
    /// Get the Greek name for the voice
    pub fn to_greek(&self) -> &'static str {
        match self {
            Voice::Active => "Ἐνεργητική",
            Voice::Middle => "Μέση",
            Voice::Passive => "Παθητική",
        }
    }
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
    fn test_greek_names_case() {
        assert_eq!(Case::Nominative.to_greek(), "Ὀνομαστική");
        assert_eq!(Case::Genitive.to_greek(), "Γενική");
        assert_eq!(Case::Dative.to_greek(), "Δοτική");
        assert_eq!(Case::Accusative.to_greek(), "Αἰτιατική");
        assert_eq!(Case::Vocative.to_greek(), "Κλητική");
    }

    #[test]
    fn test_greek_names_number() {
        assert_eq!(Number::Singular.to_greek(), "Ἑνικός");
        assert_eq!(Number::Plural.to_greek(), "Πληθυντικός");
    }

    #[test]
    fn test_greek_names_gender() {
        assert_eq!(Gender::Masculine.to_greek(), "Ἀρσενικόν");
        assert_eq!(Gender::Feminine.to_greek(), "Θηλυκόν");
        assert_eq!(Gender::Neuter.to_greek(), "Οὐδέτερον");
    }

    #[test]
    fn test_greek_names_person() {
        assert_eq!(Person::First.to_greek(), "Πρῶτον");
        assert_eq!(Person::Second.to_greek(), "Δεύτερον");
        assert_eq!(Person::Third.to_greek(), "Τρίτον");
    }

    #[test]
    fn test_greek_names_tense() {
        assert_eq!(Tense::Present.to_greek(), "Ἐνεστώς");
        assert_eq!(Tense::Imperfect.to_greek(), "Παρατατικός");
        assert_eq!(Tense::Future.to_greek(), "Μέλλων");
        assert_eq!(Tense::Aorist.to_greek(), "Ἀόριστος");
        assert_eq!(Tense::Perfect.to_greek(), "Παρακείμενος");
        assert_eq!(Tense::Pluperfect.to_greek(), "Ὑπερσυντέλικος");
    }

    #[test]
    fn test_greek_names_mood() {
        assert_eq!(Mood::Indicative.to_greek(), "Ὁριστική");
        assert_eq!(Mood::Imperative.to_greek(), "Προστακτική");
        assert_eq!(Mood::Subjunctive.to_greek(), "Ὑποτακτική");
        assert_eq!(Mood::Optative.to_greek(), "Εὐκτική");
        assert_eq!(Mood::Infinitive.to_greek(), "Ἀπαρέμφατον");
        assert_eq!(Mood::Participle.to_greek(), "Μετοχή");
    }

    #[test]
    fn test_greek_names_voice() {
        assert_eq!(Voice::Active.to_greek(), "Ἐνεργητική");
        assert_eq!(Voice::Middle.to_greek(), "Μέση");
        assert_eq!(Voice::Passive.to_greek(), "Παθητική");
    }
}
