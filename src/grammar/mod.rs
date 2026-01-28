//! Grammar parsing module using PEG grammar
//!
//! This module contains the pest grammar definition and parser for ΓΛΩΣΣΑ.

mod normalize;

pub use normalize::normalize_greek;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar/glossa.pest"]
pub struct GlossaParser;

/// Parse a ΓΛΩΣΣΑ source string into a pest parse tree
pub fn parse(source: &str) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    GlossaParser::parse(Rule::program, source)
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
}
