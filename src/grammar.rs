//! Grammar parsing module using PEG grammar
//!
//! This module is the entry point for text processing in ΓΛΩΣΣΑ.
//! It handles the initial stage of the compiler pipeline: converting raw source code
//! into a Concrete Syntax Tree (via [`pest`]) or an Abstract Syntax Tree (via the `ast` module).
//!
//! # The Parsing Pipeline
//!
//! 1. **Text Normalization**:
//!    Greek is polytonic (has accents/breathings: ἄ, ῆ, ῶ).
//!    We normalize everything to monotonic lowercase to simplify processing.
//!    `ἄνθρωπος` -> `ανθρωπος`.
//!
//! 2. **PEG Parsing** (`glossa.pest`):
//!    We use a Parsing Expression Grammar (PEG) defined in `glossa.pest`.
//!    This grammar handles the raw tokenization and structure of the language.
//!
//! 3. **AST Construction**:
//!    The `parse` function returns a `pest` Pair iterator, which is then typically
//!    converted into our AST (see `crate::ast::build_ast`).
//!
//! # Example
//!
//! ```
//! use glossa::grammar::parse;
//!
//! let source = "«χαῖρε» λέγε.";
//! let pairs = parse(source).unwrap();
//!
//! // Inspect the parse tree
//! for pair in pairs {
//!     println!("Rule: {:?}", pair.as_rule());
//!     println!("Text: {}", pair.as_str());
//! }
//! ```

use pest::Parser;
use pest_derive::Parser;
use smol_str::SmolStr;
use unicode_normalization::UnicodeNormalization;

#[derive(Parser)]
#[grammar = "glossa.pest"]
pub struct GlossaParser;

/// Parse a ΓΛΩΣΣΑ source string into a pest parse tree
///
/// This function invokes the generated PEG parser on the input source.
/// It expects a complete `program` rule as the root.
pub fn parse(source: &str) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    GlossaParser::parse(Rule::program, source)
}

// -------------------------------------------------------------------------------------------------
// Unicode Normalization
// -------------------------------------------------------------------------------------------------

/// Normalize polytonic Greek to monotonic form
///
/// This strips:
/// - Breathing marks (smooth ᾿ and rough ῾)
/// - Accents (acute ´, grave `, circumflex ῀)
/// - Iota subscript (ͅ)
/// - Diaeresis (¨)
///
/// And converts to lowercase for consistent comparison.
///
/// # Examples
///
/// ```
/// use glossa::grammar::normalize_greek;
///
/// assert_eq!(normalize_greek("ἄνθρωπος"), "ανθρωπος");
/// assert_eq!(normalize_greek("Ἀθῆναι"), "αθηναι");
/// assert_eq!(normalize_greek("χαῖρε"), "χαιρε");
/// ```
pub fn normalize_greek(text: &str) -> SmolStr {
    text.nfd() // Decompose into base + combining marks
        .filter(|c| !is_greek_diacritic(*c))
        .collect::<String>()
        .to_lowercase()
        .into()
}

/// Check if a character is a Greek diacritical mark to be stripped
fn is_greek_diacritic(c: char) -> bool {
    matches!(
        c,
        '\u{0300}'          // Combining grave accent
        | '\u{0301}'        // Combining acute accent
        | '\u{0302}'        // Combining circumflex (hat)
        | '\u{0303}'        // Combining tilde
        | '\u{0304}'        // Combining macron
        | '\u{0306}'        // Combining breve
        | '\u{0308}'        // Combining diaeresis
        | '\u{0313}'        // Combining comma above (smooth breathing)
        | '\u{0314}'        // Combining reversed comma above (rough breathing)
        | '\u{0342}'        // Combining Greek perispomeni (circumflex)
        | '\u{0343}'        // Combining Greek koronis
        | '\u{0344}'        // Combining Greek dialytika tonos
        | '\u{0345}' // Combining Greek ypogegrammeni (iota subscript)
    )
}

/// Normalize a single Greek word for lexicon lookup
#[allow(dead_code)]
pub fn normalize_word(word: &str) -> SmolStr {
    normalize_greek(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hello_cosmos() {
        let source = "«χαῖρε κόσμε» λέγε.";
        let result = parse(source);
        assert!(
            result.is_ok(),
            "Failed to parse hello cosmos: {:?}",
            result.err()
        );

        let pairs = result.unwrap();
        // Verify we got a program with at least one statement
        let program = pairs.into_iter().next().unwrap();
        assert_eq!(program.as_rule(), Rule::program);
    }

    #[test]
    fn test_parse_simple_string_print() {
        let source = "«χαῖρε» λέγε.";
        let result = parse(source);
        assert!(
            result.is_ok(),
            "Failed to parse simple print: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_variable_binding() {
        let source = "ξ πέντε ἔστω.";
        let result = parse(source);
        assert!(
            result.is_ok(),
            "Failed to parse variable binding: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_variable_use() {
        let source = "ξ λέγε.";
        let result = parse(source);
        assert!(
            result.is_ok(),
            "Failed to parse variable use: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_multiple_statements() {
        let source = "ξ πέντε ἔστω. ξ λέγε.";
        let result = parse(source);
        assert!(
            result.is_ok(),
            "Failed to parse multiple statements: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_number_literal() {
        let source = "42 λέγε.";
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse number: {:?}", result.err());
    }

    #[test]
    fn test_parse_genitive_property_access() {
        // "the name of the user" - genitive shows possession
        let source = "χρήστου ὄνομα λέγε.";
        let result = parse(source);
        assert!(
            result.is_ok(),
            "Failed to parse genitive: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_chained_statements() {
        // Using ano teleia (· U+00B7) to chain - the Greek semicolon
        let source = "«χαῖρε» λέγε· «κόσμε» λέγε.";
        let result = parse(source);
        assert!(
            result.is_ok(),
            "Failed to parse chained statements: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_greek_question_mark() {
        // Using Greek question mark (U+037E) - looks like ; but is different
        let source = "ξ\u{037E}"; // "what is ξ?"
        let result = parse(source);
        assert!(
            result.is_ok(),
            "Failed to parse Greek question mark: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_ascii_question_mark() {
        // ASCII ? also works for convenience
        let source = "ξ?";
        let result = parse(source);
        assert!(
            result.is_ok(),
            "Failed to parse ASCII question mark: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_line_comment() {
        // Comments use // like Rust
        let source = "// τοῦτο σχόλιόν ἐστι\n«χαῖρε» λέγε.";
        let result = parse(source);
        assert!(
            result.is_ok(),
            "Failed to parse comment: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_inline_comment() {
        let source = "ξ πέντε ἔστω. // binding ξ to 5\nξ λέγε.";
        let result = parse(source);
        assert!(
            result.is_ok(),
            "Failed to parse inline comment: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_normalize_anthropos() {
        assert_eq!(normalize_greek("ἄνθρωπος"), "ανθρωπος");
    }

    #[test]
    fn test_normalize_athenai() {
        assert_eq!(normalize_greek("Ἀθῆναι"), "αθηναι");
    }

    #[test]
    fn test_normalize_chaire() {
        assert_eq!(normalize_greek("χαῖρε"), "χαιρε");
    }

    #[test]
    fn test_normalize_lege() {
        assert_eq!(normalize_greek("λέγε"), "λεγε");
    }

    #[test]
    fn test_normalize_esto() {
        assert_eq!(normalize_greek("ἔστω"), "εστω");
    }

    #[test]
    fn test_normalize_rough_breathing() {
        // Rough breathing on initial vowel
        assert_eq!(normalize_greek("ὁ"), "ο");
        assert_eq!(normalize_greek("ἡ"), "η");
    }

    #[test]
    fn test_normalize_iota_subscript() {
        // ᾳ = α + iota subscript
        assert_eq!(normalize_greek("τῇ"), "τη");
        assert_eq!(normalize_greek("τῷ"), "τω");
    }

    #[test]
    fn test_normalize_circumflex() {
        assert_eq!(normalize_greek("κῆπος"), "κηπος");
        assert_eq!(normalize_greek("δῶρον"), "δωρον");
    }

    #[test]
    fn test_normalize_preserves_base_letters() {
        // Should not change letters without diacritics
        assert_eq!(normalize_greek("κοσμος"), "κοσμος");
        assert_eq!(normalize_greek("λογος"), "λογος");
    }

    #[test]
    fn test_normalize_mixed_text() {
        // String with Greek and punctuation
        assert_eq!(normalize_greek("«χαῖρε κόσμε»"), "«χαιρε κοσμε»");
    }

    #[test]
    fn test_normalize_genitive_ending() {
        assert_eq!(normalize_greek("χρήστου"), "χρηστου");
        assert_eq!(normalize_greek("λόγου"), "λογου");
    }

    #[test]
    fn test_normalize_case_insensitive() {
        assert_eq!(normalize_greek("ΛΟΓΟΣ"), "λογος");
        assert_eq!(normalize_greek("Λόγος"), "λογος");
    }

    #[test]
    fn test_normalize_meizon() {
        // μεῖζον with circumflex should normalize to μειζον
        assert_eq!(normalize_greek("μεῖζον"), "μειζον");
    }

    #[test]
    fn test_normalize_or_particle() {
        // ἤ (eta with smooth breathing + acute) should normalize to η
        assert_eq!(normalize_greek("ἤ"), "η");
    }

    #[test]
    fn test_normalize_subjunctive_eimi() {
        // ᾖ (eta with iota subscript + circumflex) should normalize to η
        assert_eq!(normalize_greek("ᾖ"), "η");
    }
}
