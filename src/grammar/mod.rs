//! Grammar parsing module using PEG grammar
//!
//! This module is the entry point for text processing in ΓΛΩΣΣΑ.
//! It handles the initial stage of the compiler pipeline: converting raw source code
//! into a Concrete Syntax Tree (via [`pest`]) or an Abstract Syntax Tree (via the `ast` module).
//!
//! # The Grammar (`glossa.pest`)
//!
//! The language syntax is defined in [glossa.pest](https://github.com/madmax983/glossa/blob/trunk/src/grammar/glossa.pest).
//!
//! ## High-Level Structure
//!
//! * **Program**: A sequence of `Statement`s.
//! * **Statement**: Can be a `TypeDefinition`, `TraitDefinition`, `TraitImplementation`, or a `Regular` statement.
//! * **Regular Statement**: Composed of lists of expressions (clauses) separated by commas.
//! * **Clause**: A sequence of `Expression`s (represented as `Vec<Expr>`).
//! * **Expression**: Words, literals, phrases, or blocks.
//!
//! # The Parsing Pipeline
//!
//! 1. **Text Normalization** (`text` module):
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

#[derive(Parser)]
#[grammar = "grammar/glossa.pest"]
pub struct GlossaParser;

/// Parse a ΓΛΩΣΣΑ source string into a pest parse tree
///
/// This function invokes the generated PEG parser on the input source.
/// It expects a complete `program` rule as the root.
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
