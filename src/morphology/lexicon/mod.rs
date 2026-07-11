pub mod data;
pub use data::*;

use crate::morphology::models::PartOfSpeech;

/// Check if a word is a known verb
pub fn is_verb(normalized_word: &str) -> bool {
    lookup(normalized_word)
        .map(|e| e.pos == PartOfSpeech::Verb)
        .unwrap_or(false)
}

/// Check if a word is a binding verb (ἔστω / ἔστωσαν / εἰμί)
/// This includes the conjugated forms (εστω, εστωσαν) and the lemma (ειμι)
pub fn is_binding_verb(normalized_word: &str) -> bool {
    matches!(
        normalized_word,
        "εστω" | "εστωσαν" | "ειμι" | "εστι" | "εισι" | "ειναι"
    )
}

/// Check if a word is a print verb (λέγε, γράφε)
pub fn is_print_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "λεγε" | "γραφε" | "λεγω" | "γραφω")
}

/// Check if a word is a find verb (εὑρέ)
pub fn is_find_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "ευρε" | "ευρισκω")
}

/// Check if a word is the δεῖ assertion verb
pub fn is_assert_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "δει" | "dei")
}

/// Check if a word is the ἰσοῦται equality verb
pub fn is_equals_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "ισοω" | "isoo" | "ισουται" | "isoutai")
}

/// Check if a word is the "any" quantifier (τι)
pub fn is_any_quantifier(normalized_word: &str) -> bool {
    matches!(normalized_word, "τι" | "τις") // τι is the form, τις is the lemma
}

/// Check if a word is the "all" quantifier (πάντα)
pub fn is_all_quantifier(normalized_word: &str) -> bool {
    matches!(normalized_word, "παντα" | "πας") // πάντα is the form, πας is the lemma
}

/// Check if a word is a mutable marker (μετά)
pub fn is_mutable_marker(normalized_word: &str) -> bool {
    normalized_word == "μετα"
}

/// Check if a word is an assignment verb (γίγνεται / γίγνομαι)
pub fn is_assignment_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "γιγνεται" | "γιγνομαι")
}

/// Get the numeric value of a Greek numeral word
/// Includes all case forms (nominative, genitive, dative, accusative)
pub fn numeral_value(normalized_word: &str) -> Option<i64> {
    match normalized_word {
        // 0 - μηδέν (nothing/zero)
        "μηδεν" | "μηδενος" => Some(0),
        // 1 - εἷς, μία, ἕν
        "εν" | "ενα" | "ενος" | "μια" | "μιας" => Some(1),
        // 2 - δύο (indeclinable)
        "δυο" | "δυοιν" => Some(2),
        // 3 - τρεῖς, τρία
        "τρια" | "τρεις" | "τριων" | "τρισι" | "τρισιν" => Some(3),
        // 4 - τέτταρες, τέτταρα
        "τεσσαρα" | "τεσσαρες" | "τεσσαρων" | "τεσσαρσι" | "τεσσαρσιν" => {
            Some(4)
        }
        // 5 - πέντε (indeclinable)
        "πεντε" => Some(5),
        // 6 - ἕξ (indeclinable)
        "εξ" => Some(6),
        // 7 - ἑπτά (indeclinable)
        "επτα" => Some(7),
        // 8 - ὀκτώ (indeclinable)
        "οκτω" => Some(8),
        // 9 - ἐννέα (indeclinable)
        "εννεα" => Some(9),
        // 10 - δέκα (indeclinable)
        "δεκα" => Some(10),
        // 100 - ἑκατόν (indeclinable)
        "εκατον" => Some(100),
        // 1000 - χίλιοι/χίλια
        "χιλια" | "χιλιοι" | "χιλιων" => Some(1000),
        _ => None,
    }
}

/// Binary operator type for code generation
///
/// In ΓΛΩΣΣΑ, operators are frequently expressed as descriptive adjectives or nouns
/// (e.g. `μείζον` for "greater than", `ἄθροισμα` for "sum"). These concepts are mapped
/// to fundamental binary operations during the assembly phase, establishing relations
/// between two distinct expressions.
///
/// ## Examples
///
/// ```rust
/// use glossa::morphology::{BinaryOp, comparison_operator};
///
/// // The Greek word for "equal" (ἴσον) translates directly to the Eq binary operator
/// let op = comparison_operator("ισον").unwrap();
/// assert_eq!(op, BinaryOp::Eq);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    /// Synthesizes two values into their combined magnitude (`ἄθροισμα`).
    Add,
    /// Evaluates the quantitative difference between two values (`διαφορά`).
    Sub,
    /// Expands the magnitude of a value by the factor of another (`γινόμενον`).
    Mul,
    /// Partitions a value into equal segments (`μέρος`).
    Div,
    /// Isolates the remainder left over from a division (`ὑπόλοιπον`).
    Mod,
    // Comparison
    /// Tests if the identities of two values perfectly align (`ἴσον`).
    Eq,
    /// Tests if the identities of two values are distinct (`ἄνισον`).
    Ne,
    /// Asserts the preceding value is quantitatively lesser (`ἔλαττον`).
    Lt,
    /// Asserts the preceding value is bounded by the subsequent value.
    Le,
    /// Asserts the preceding value is quantitatively greater (`μεῖζον`).
    Gt,
    /// Asserts the preceding value dominates or equates to the subsequent value.
    Ge,
    // Boolean
    /// Conjoins two truths, requiring both to manifest reality (`καί`).
    And,
    /// Offers an alternative path, requiring only one truth to manifest reality (`ἤ`).
    Or,
}

impl BinaryOp {}

/// Unary operator type for code generation
///
/// Modifies a single expression's existential state or reference.
///
/// ## Examples
///
/// ```rust
/// use glossa::morphology::{UnaryOp, is_negation};
///
/// // The Greek particle "οὐ" negates existence
/// assert!(is_negation("ου"));
/// let not_op = UnaryOp::Not;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Inverts the truth of a statement, stemming from the absolute negation particle (`οὐ`/`οὐκ`).
    Not,
    /// Flips the quantitative sign of an arithmetic value.
    Neg,
    /// Establishes an indirect relationship to an entity, denoting a view rather than ownership.
    Ref,
}

impl UnaryOp {}

/// Check if a word is a comparison adjective and return the operator
pub fn comparison_operator(normalized_word: &str) -> Option<BinaryOp> {
    match normalized_word {
        "μειζον" => Some(BinaryOp::Gt),
        "ελαττον" => Some(BinaryOp::Lt),
        "ισον" => Some(BinaryOp::Eq),
        "ανισον" => Some(BinaryOp::Ne),
        _ => None,
    }
}

/// Check if a word is a boolean conjunction and return the operator
pub fn boolean_operator(normalized_word: &str) -> Option<BinaryOp> {
    match normalized_word {
        "και" => Some(BinaryOp::And),
        "η" => Some(BinaryOp::Or),
        _ => None,
    }
}

/// Check if a word is a negation particle
pub fn is_negation(normalized_word: &str) -> bool {
    matches!(normalized_word, "ου" | "ουκ" | "ουχ")
}

/// Check if a word is an arithmetic noun and return the operator
pub fn arithmetic_operator(normalized_word: &str) -> Option<BinaryOp> {
    match normalized_word {
        "αθροισμα" => Some(BinaryOp::Add),
        "διαφορα" => Some(BinaryOp::Sub),
        "γινομενον" => Some(BinaryOp::Mul),
        "μερος" => Some(BinaryOp::Div),
        "υπολοιπον" => Some(BinaryOp::Mod),
        _ => None,
    }
}

/// Check if a word maps to any binary operator
pub fn is_operator_word(normalized_word: &str) -> bool {
    comparison_operator(normalized_word).is_some()
        || boolean_operator(normalized_word).is_some()
        || arithmetic_operator(normalized_word).is_some()
}

/// Get the binary operator for any operator word
pub fn get_binary_operator(normalized_word: &str) -> Option<BinaryOp> {
    comparison_operator(normalized_word)
        .or_else(|| boolean_operator(normalized_word))
        .or_else(|| arithmetic_operator(normalized_word))
}

// =============================================================================
// Control Flow Particles (Phase 2)
// =============================================================================

/// Check if a word is a conditional particle (if)
pub fn is_conditional_particle(normalized_word: &str) -> bool {
    matches!(normalized_word, "ει" | "εαν" | "ην" | "αν")
}

/// The canonical sequence of words defining an 'else' block in ΓΛΩΣΣΑ logic.
///
/// This specific sequence of particles and negations ("εἰ", "δὲ", "μή") serves as
/// the definitive pattern for fallback execution paths when prior conditions fail.
///
/// # Why these exact words?
/// In Classical Greek conditional clauses, "εἰ δὲ μή" (literally "but if not")
/// elegantly captures the semantics of an 'otherwise' or 'else' branch without
/// requiring a dedicated keyword.
///
/// # Examples
/// ```
/// use glossa::morphology::ELSE_PATTERN_WORDS;
///
/// assert_eq!(ELSE_PATTERN_WORDS, ["ει", "δε", "μη"]);
/// ```
pub const ELSE_PATTERN_WORDS: [&str; 3] = ["ει", "δε", "μη"];

/// Check if a sequence is the else pattern (εἰ δὲ μή)
pub fn is_else_pattern(normalized_phrase: &str) -> bool {
    normalized_phrase == "ει δε μη"
}

/// Check if a word is a loop particle
pub fn is_loop_particle(normalized_word: &str) -> bool {
    matches!(normalized_word, "εως" | "δια")
}

/// Check if a word is a range particle
pub fn is_range_particle(normalized_word: &str) -> bool {
    matches!(normalized_word, "απο" | "μεχρι" | "εως")
}

/// Check if a word is the break verb (παῦε)
pub fn is_break_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "παυε" | "παυω")
}

/// Check if a word is the continue verb (συνέχιζε)
pub fn is_continue_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "συνεχιζε" | "συνεχιζω")
}

/// Check if a word is the match particle (κατά)
pub fn is_match_particle(normalized_word: &str) -> bool {
    normalized_word == "κατα"
}

// =============================================================================
// Collection Operations (Phase 3)
// =============================================================================

/// Check if a word is a push verb (ὠθεῖ/ὠθέω)
pub fn is_push_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "ωθει" | "ωθεω" | "ωθω")
}

/// Check if a word is a pop verb (ἕλκεται/ἕλκομαι - middle voice)
pub fn is_pop_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "ελκεται" | "ελκομαι" | "ελκω")
}

/// Check if a word is the length property (μῆκος)
pub fn is_length_property(normalized_word: &str) -> bool {
    normalized_word == "μηκος"
}

/// Check if a word is an ordinal adjective
pub fn is_ordinal(normalized_word: &str) -> bool {
    matches!(
        normalized_word,
        "πρωτον"
            | "πρωτου"
            | "δευτερον"
            | "δευτερου"
            | "τριτον"
            | "τριτου"
            | "τεταρτον"
            | "τεταρτου"
            | "πεμπτον"
            | "πεμπτου"
    )
}

/// Convert an ordinal word to a zero-based array index
/// Returns None if the word is not a recognized ordinal
pub fn ordinal_to_index(normalized_word: &str) -> Option<i64> {
    match normalized_word {
        "πρωτον" | "πρωτου" => Some(0),     // first = 0
        "δευτερον" | "δευτερου" => Some(1), // second = 1
        "τριτον" | "τριτου" => Some(2),     // third = 2
        "τεταρτον" | "τεταρτου" => Some(3), // fourth = 3
        "πεμπτον" | "πεμπτου" => Some(4),   // fifth = 4
        _ => None,
    }
}

/// Check if a word is a necessity particle (ἀνάγκη - necessarily)
/// This is the Aristotelian marker for logical consequence
pub fn is_necessity_particle(normalized_word: &str) -> bool {
    matches!(normalized_word, "αναγκη" | "αναγκαιον")
}

/// Check if a word is an obligation particle (δεῖ - must/ought)
/// Less absolute than ἀνάγκη, used for practical necessity
pub fn is_obligation_particle(normalized_word: &str) -> bool {
    matches!(normalized_word, "δει" | "δεον" | "χρη")
}

/// Check if a word introduces a consequence (ἀνάγκη or δεῖ)
pub fn is_consequence_marker(normalized_word: &str) -> bool {
    is_necessity_particle(normalized_word) || is_obligation_particle(normalized_word)
}

/// Check if a word represents None (οὐδέν)
pub fn is_none_word(normalized_word: &str) -> bool {
    matches!(normalized_word, "ουδεν")
}

/// Check if a word represents Some (τί)
pub fn is_some_word(normalized_word: &str) -> bool {
    matches!(normalized_word, "τι")
}

/// Check if a word represents Ok (ἐπιτυχία)
pub fn is_ok_word(normalized_word: &str) -> bool {
    matches!(normalized_word, "επιτυχια")
}

/// Check if a word represents Err (σφάλμα)
pub fn is_err_word(normalized_word: &str) -> bool {
    matches!(normalized_word, "σφαλμα")
}

// =============================================================================
// HashMap/HashSet/String Operations (Issue #77)
// =============================================================================

/// Check if a word is an insert verb (τίθημι - to place/insert)
/// Maps to `.insert()` for HashMap/HashSet
pub fn is_insert_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "τιθησι" | "τιθημι" | "θες")
}

/// Check if a word is a split verb (σχίζω - to split)
/// Maps to `.split()` for String
pub fn is_split_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "σχιζει" | "σχιζεται" | "σχιζω")
}

/// Check if a word is a join verb (ἑνόω - to unite/join)
/// Maps to `.join()` for iterators of strings
pub fn is_join_verb(normalized_word: &str) -> bool {
    matches!(normalized_word, "ενουνται" | "ενουσι" | "ενοω")
}

/// Check if a word is a containment preposition (ἐν - in)
/// Used for `.contains()` and `.contains_key()` patterns
pub fn is_containment_preposition(normalized_word: &str) -> bool {
    normalized_word == "εν"
}

/// Check if a word is a delimiter preposition (κατά - according to/by)
/// Used for `.split()` and `.join()` patterns
pub fn is_delimiter_preposition(normalized_word: &str) -> bool {
    normalized_word == "κατα" || normalized_word == "κατ"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphology::models::{Case, Mood, Number, Person, Tense, Voice};

    #[test]
    fn test_lookup_verb() {
        let entry = lookup("λεγε").unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Verb);
        assert_eq!(entry.mood, Some(Mood::Imperative));
    }

    #[test]
    fn test_lookup_binding() {
        let entry = lookup("εστω").unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Verb);
        assert!(is_binding_verb("εστω"));
    }

    #[test]
    fn test_lookup_type() {
        let entry = lookup("αριθμος").unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Noun);
        assert_eq!(entry.rust_equiv, Some("i64"));
    }

    #[test]
    fn test_is_print_verb() {
        assert!(is_print_verb("λεγε"));
        assert!(is_print_verb("γραφε"));
        assert!(!is_print_verb("εστω"));
    }

    #[test]
    fn test_numeral_value() {
        assert_eq!(numeral_value("πεντε"), Some(5));
        assert_eq!(numeral_value("δεκα"), Some(10));
        assert_eq!(numeral_value("foo"), None);
    }

    #[test]
    fn test_boolean_lookup() {
        let entry = lookup("αληθες").unwrap();
        assert_eq!(entry.rust_equiv, Some("true"));

        let entry = lookup("ψευδος").unwrap();
        assert_eq!(entry.rust_equiv, Some("false"));
    }

    #[test]
    fn test_comparison_operators() {
        assert_eq!(comparison_operator("μειζον"), Some(BinaryOp::Gt));
        assert_eq!(comparison_operator("ελαττον"), Some(BinaryOp::Lt));
        assert_eq!(comparison_operator("ισον"), Some(BinaryOp::Eq));
        assert_eq!(comparison_operator("ανισον"), Some(BinaryOp::Ne));
        assert_eq!(comparison_operator("foo"), None);
    }

    #[test]
    fn test_boolean_operators() {
        assert_eq!(boolean_operator("και"), Some(BinaryOp::And));
        assert_eq!(boolean_operator("η"), Some(BinaryOp::Or));
        assert!(is_negation("ου"));
        assert!(is_negation("ουκ"));
        assert!(is_negation("ουχ"));
    }

    #[test]
    fn test_arithmetic_operators() {
        assert_eq!(arithmetic_operator("αθροισμα"), Some(BinaryOp::Add));
        assert_eq!(arithmetic_operator("διαφορα"), Some(BinaryOp::Sub));
        assert_eq!(arithmetic_operator("γινομενον"), Some(BinaryOp::Mul));
        assert_eq!(arithmetic_operator("μερος"), Some(BinaryOp::Div));
        assert_eq!(arithmetic_operator("υπολοιπον"), Some(BinaryOp::Mod));
    }

    #[test]
    fn test_operator_lexicon_entries() {
        // Comparison adjectives
        let entry = lookup("μειζον").unwrap();
        assert_eq!(entry.rust_equiv, Some(">"));
        assert_eq!(entry.pos, PartOfSpeech::Adjective);

        // Boolean particles
        let entry = lookup("και").unwrap();
        assert_eq!(entry.rust_equiv, Some("&&"));
        assert_eq!(entry.pos, PartOfSpeech::Conjunction);

        // Arithmetic nouns
        let entry = lookup("αθροισμα").unwrap();
        assert_eq!(entry.rust_equiv, Some("+"));
        assert_eq!(entry.pos, PartOfSpeech::Noun);
    }

    #[test]
    fn test_meta_is_mutable_marker() {
        assert!(is_mutable_marker("μετα"));
    }

    #[test]
    fn test_meta_lexicon_entry() {
        let entry = lookup("μετα").unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Preposition);
        assert_eq!(entry.rust_equiv, Some("mut"));
    }

    #[test]
    fn test_non_mutable_marker() {
        assert!(!is_mutable_marker("εστω"));
        assert!(!is_mutable_marker("λεγε"));
    }

    #[test]
    fn test_gignetai_is_assignment_verb() {
        assert!(is_assignment_verb("γιγνεται"));
        assert!(is_assignment_verb("γιγνομαι"));
    }

    #[test]
    fn test_gignetai_lexicon_entry() {
        let entry = lookup("γιγνεται").unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Verb);
        assert_eq!(entry.voice, Some(Voice::Middle));
        assert_eq!(entry.lemma, "γιγνομαι");
    }

    #[test]
    fn test_non_assignment_verb() {
        assert!(!is_assignment_verb("εστω"));
        assert!(!is_assignment_verb("λεγε"));
    }

    // =========================================================================
    // HashMap/HashSet/String operation tests (Issue #77)
    // =========================================================================

    #[test]
    fn test_is_insert_verb() {
        assert!(is_insert_verb("τιθησι"));
        assert!(is_insert_verb("τιθημι"));
        assert!(is_insert_verb("θες"));
        assert!(!is_insert_verb("λεγε"));
        assert!(!is_insert_verb("ωθει"));
    }

    #[test]
    fn test_is_split_verb() {
        assert!(is_split_verb("σχιζει"));
        assert!(is_split_verb("σχιζεται"));
        assert!(is_split_verb("σχιζω"));
        assert!(!is_split_verb("λεγε"));
    }

    #[test]
    fn test_is_join_verb() {
        assert!(is_join_verb("ενουνται"));
        assert!(is_join_verb("ενουσι"));
        assert!(is_join_verb("ενοω"));
        assert!(!is_join_verb("λεγε"));
    }

    #[test]
    fn test_is_containment_preposition() {
        assert!(is_containment_preposition("εν"));
        assert!(!is_containment_preposition("δια"));
        assert!(!is_containment_preposition("εις"));
    }

    #[test]
    fn test_is_delimiter_preposition() {
        assert!(is_delimiter_preposition("κατα"));
        assert!(is_delimiter_preposition("κατ"));
        assert!(!is_delimiter_preposition("εν"));
        assert!(!is_delimiter_preposition("δια"));
    }

    #[test]
    fn test_insert_verb_lexicon_entries() {
        let entry = lookup("τιθησι").unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Verb);
        assert_eq!(entry.rust_equiv, Some(".insert"));
        assert_eq!(entry.lemma, "τιθημι");

        let entry = lookup("θες").unwrap();
        assert_eq!(entry.mood, Some(Mood::Imperative));
    }

    #[test]
    fn test_split_verb_lexicon_entries() {
        let entry = lookup("σχιζεται").unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Verb);
        assert_eq!(entry.rust_equiv, Some(".split"));
        assert_eq!(entry.voice, Some(Voice::Middle));
    }

    #[test]
    fn test_join_verb_lexicon_entries() {
        let entry = lookup("ενουνται").unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Verb);
        assert_eq!(entry.rust_equiv, Some(".join"));
        assert_eq!(entry.voice, Some(Voice::Middle));
    }

    #[test]
    fn test_declined_collection_nouns() {
        // HashMap declined forms
        let entry = lookup("χαρτη").unwrap();
        assert_eq!(entry.case, Some(Case::Dative));
        assert_eq!(entry.lemma, "χαρτης");

        let entry = lookup("χαρτου").unwrap();
        assert_eq!(entry.case, Some(Case::Genitive));

        // HashSet declined forms
        let entry = lookup("συνολω").unwrap();
        assert_eq!(entry.case, Some(Case::Dative));
        assert_eq!(entry.lemma, "συνολον");

        // String declined forms
        let entry = lookup("λογω").unwrap();
        assert_eq!(entry.case, Some(Case::Dative));
        assert_eq!(entry.lemma, "λογος");
    }

    #[test]
    fn test_is_assert_verb() {
        assert!(is_assert_verb("δει"));
        assert!(is_assert_verb("dei"));
        assert!(!is_assert_verb("εστω"));
        assert!(!is_assert_verb("λεγε"));
    }

    #[test]
    fn test_is_equals_verb() {
        assert!(is_equals_verb("ισοω"));
        assert!(is_equals_verb("isoo"));
        assert!(is_equals_verb("ισουται"));
        assert!(is_equals_verb("isoutai"));
        assert!(!is_equals_verb("δει"));
        assert!(!is_equals_verb("εστω"));
    }

    #[test]
    fn test_assert_verb_lexicon_entry() {
        let entry = lookup("δει").unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Verb);
        assert_eq!(entry.rust_equiv, Some("assert!"));
        assert_eq!(entry.lemma, "δει");
        assert_eq!(entry.person, Some(Person::Third));
        assert_eq!(entry.number, Some(Number::Singular));
    }

    #[test]
    fn test_equals_verb_lexicon_entry() {
        let entry = lookup("ισουται").unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Verb);
        assert_eq!(entry.rust_equiv, Some("assert_eq!"));
        assert_eq!(entry.lemma, "ισοω");
        assert_eq!(entry.voice, Some(Voice::Middle));
        assert_eq!(entry.person, Some(Person::Third));
        assert_eq!(entry.number, Some(Number::Singular));
    }
}
