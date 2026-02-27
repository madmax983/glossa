//! Semantic Parser for ΓΛΩΣΣΑ
//!
//! This module bridges the gap between the raw text parsing (Grammar) and the
//! high-level program structure (AST).
//!
//! # The Parsing Flow
//!
//! 1. **Grammar (`src/grammar`)**: Uses [`pest`] (PEG parser) to tokenize the input
//!    and verify it matches the language rules. This produces a "Concrete Syntax Tree" (CST)
//!    of untyped pairs (e.g., `Rule::greek_word`, `Rule::number_literal`).
//!
//! 2. **Builder**: Walks this CST and constructs strongly-typed
//!    [`crate::ast`] nodes. This is where we handle:
//!    * Text normalization (converting `Ἀθῆναι` to `αθηναι`)
//!    * Number parsing
//!    * Structural validation (e.g., ensuring a trait method has a name)
//!
//! # Safety: Recursion Depth
//!
//! ΓΛΩΣΣΑ implements a strict recursion depth check (`check_recursion_depth`)
//! before parsing begins. This linear scan of the source code ensures that deep
//! nesting (e.g., `((((...))))`) does not cause a stack overflow during the
//! recursive descent parsing phase.

pub(crate) mod common;
pub(crate) mod declarations;
pub(crate) mod expressions;
pub mod grammar;
pub mod numerals;
pub mod recursion;
pub(crate) mod statements;

use self::grammar::{Rule, parse as grammar_parse};
use crate::ast::*;
use crate::errors::GlossaError;
use pest::iterators::Pair;

pub use common::ParseError;

impl From<ParseError> for GlossaError {
    fn from(err: ParseError) -> Self {
        match err {
            ParseError::PestError { message, span } => GlossaError::parse_with_source(
                message,
                String::new(),
                miette::SourceSpan::new(span.0.into(), span.1),
            ),
            _ => GlossaError::parse(err.to_string()),
        }
    }
}

/// Parse a ΓΛΩΣΣΑ source string into an AST
///
/// This is the main entry point for the parsing phase.
///
/// # Examples
///
/// ```
/// use glossa::parser::parse;
///
/// let source = "«χαῖρε» λέγε.";
/// let program = parse(source).unwrap();
/// assert_eq!(program.statements.len(), 1);
/// ```
pub fn parse(source: &str) -> Result<Program, GlossaError> {
    match parse_source(source) {
        Ok(program) => Ok(program),
        Err(ParseError::PestError { message, span }) => Err(GlossaError::parse_with_source(
            message,
            source.to_string(),
            miette::SourceSpan::new(span.0.into(), span.1),
        )),
        Err(e) => Err(GlossaError::from(e)),
    }
}

/// Build an AST from source code
fn parse_source(source: &str) -> Result<Program, ParseError> {
    // Check recursion depth before parsing to prevent stack overflow
    recursion::check_recursion_depth(source)?;

    let pairs = grammar_parse(source).map_err(|e| match e.line_col {
        pest::error::LineColLocation::Pos((line, col)) => {
            let offset = match e.location {
                pest::error::InputLocation::Pos(o) => o,
                pest::error::InputLocation::Span((start, _)) => start,
            };
            ParseError::PestError {
                message: format!("Parse error at {}:{}", line, col),
                span: (offset, 1),
            }
        }
        pest::error::LineColLocation::Span((start_line, start_col), _) => {
            let (start, end) = match e.location {
                pest::error::InputLocation::Pos(o) => (o, o + 1),
                pest::error::InputLocation::Span((s, e)) => (s, e),
            };
            ParseError::PestError {
                message: format!("Parse error at {}:{}", start_line, start_col),
                span: (start, end - start),
            }
        }
    })?;

    let mut statements = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::statement {
                    statements.push(build_statement(inner)?);
                }
            }
        }
    }

    Ok(Program { statements })
}

pub(crate) fn build_statement(pair: Pair<'_, Rule>) -> Result<Statement, ParseError> {
    let mut pairs = pair.into_inner();
    let first = pairs
        .next()
        .ok_or(ParseError::UnexpectedRule("Empty statement".into()))?;

    match first.as_rule() {
        Rule::test_declaration => Ok(Statement::TestDeclaration(
            declarations::build_test_declaration(first)?,
        )),
        Rule::type_definition => {
            // Consume statement_end
            let _ = pairs.next();
            Ok(Statement::TypeDefinition(
                declarations::build_type_definition(first)?,
            ))
        }
        Rule::trait_definition => {
            // Consume statement_end
            let _ = pairs.next();
            Ok(Statement::TraitDefinition(
                declarations::build_trait_definition(first)?,
            ))
        }
        Rule::trait_impl => {
            // Consume statement_end
            let _ = pairs.next();
            Ok(Statement::TraitImpl(declarations::build_trait_impl(first)?))
        }
        Rule::clause_list => statements::build_regular_statement(first, pairs),
        _ => Err(ParseError::UnexpectedRule(format!(
            "Unexpected start of statement: {:?}",
            first.as_rule()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to get the first expression of the first clause
    fn first_expr(stmt: &Statement) -> &Expr {
        match stmt {
            Statement::Regular { clauses, .. } => &clauses[0].expressions[0],
            Statement::TypeDefinition(_) => panic!("Cannot get first_expr from TypeDefinition"),
            Statement::TraitDefinition(_) => panic!("Cannot get first_expr from TraitDefinition"),
            Statement::TraitImpl(_) => panic!("Cannot get first_expr from TraitImpl"),
            Statement::TestDeclaration(_) => panic!("Cannot get first_expr from TestDeclaration"),
        }
    }

    #[test]
    fn test_parse_error_conversion() {
        // This test ensures the From<ParseError> for GlossaError impl is covered
        let err = ParseError::PestError {
            message: "Test Error".to_string(),
            span: (0, 5),
        };
        let glossa_err: GlossaError = err.into();
        let msg = glossa_err.to_string();
        assert!(msg.contains("Test Error"));
        // The From impl provides empty source string, so context might be limited,
        // but it should be a ParseError variant.
        assert!(matches!(glossa_err, GlossaError::ParseError { .. }));
    }

    #[test]
    fn test_parse_source_hello() {
        let source = "«χαῖρε» λέγε.";
        let ast = parse(source).unwrap();

        assert_eq!(ast.statements.len(), 1);
        assert!(!ast.statements[0].is_query());
    }

    #[test]
    fn test_parse_source_string_literal() {
        let source = "«χαῖρε κόσμε» λέγε.";
        let ast = parse(source).unwrap();

        let expr = first_expr(&ast.statements[0]);
        if let Expr::Phrase(terms) = expr {
            assert!(matches!(&terms[0], Expr::StringLiteral(s) if s == "χαῖρε κόσμε"));
        } else {
            panic!("Expected Phrase, got {:?}", expr);
        }
    }

    #[test]
    fn test_parse_source_variable_binding() {
        let source = "ξ πέντε ἔστω.";
        let ast = parse(source).unwrap();

        assert_eq!(ast.statements.len(), 1);
        let expr = first_expr(&ast.statements[0]);

        // Should have three words: ξ, πέντε, ἔστω
        if let Expr::Phrase(terms) = expr {
            assert_eq!(terms.len(), 3);
            if let Expr::Word(w) = &terms[0] {
                assert_eq!(w.normalized, "ξ");
            }
        } else {
            panic!("Expected Phrase");
        }
    }

    #[test]
    fn test_parse_source_number_literal() {
        let source = "42 λέγε.";
        let ast = parse(source).unwrap();

        let expr = first_expr(&ast.statements[0]);
        if let Expr::Phrase(terms) = expr {
            assert!(matches!(&terms[0], Expr::NumberLiteral(42)));
        }
    }

    #[test]
    fn test_parse_source_query() {
        let source = "ξ?";
        let ast = parse(source).unwrap();

        assert!(ast.statements[0].is_query());
    }

    #[test]
    fn test_parse_source_multiple_statements() {
        let source = "ξ πέντε ἔστω. ξ λέγε.";
        let ast = parse(source).unwrap();

        assert_eq!(ast.statements.len(), 2);
    }

    #[test]
    fn test_word_normalization() {
        let source = "χρήστου ὄνομα λέγε.";
        let ast = parse(source).unwrap();

        let expr = first_expr(&ast.statements[0]);
        if let Expr::Phrase(terms) = expr
            && let Expr::Word(w) = &terms[0]
        {
            assert_eq!(w.original, "χρήστου");
            assert_eq!(w.normalized, "χρηστου");
        }
    }

    #[test]
    fn test_parse_source_with_comma() {
        // Test that commas create multiple clauses
        let source = "εἰ ξ μεῖζον, «ναί» λέγε.";
        let ast = parse(source).unwrap();

        assert_eq!(ast.statements.len(), 1);
        assert_eq!(ast.statements[0].clauses().len(), 2); // Two clauses separated by comma
    }

    #[test]
    fn test_recursion_limit_exceeded() {
        // 501 nested parentheses
        let source = "(".repeat(501) + &")".repeat(501);
        let result = parse_source(&source);
        assert!(matches!(
            result,
            Err(ParseError::RecursionLimitExceeded(500))
        ));
    }

    #[test]
    fn test_recursion_limit_not_exceeded() {
        // 500 nested parentheses (should pass check, though pest might fail to parse empty parens)
        let source = "(".repeat(500) + &")".repeat(500);
        // We only care about the recursion check here
        let result = recursion::check_recursion_depth(&source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recursion_limit_ignored_in_string() {
        // Parentheses inside string literal shouldn't count
        let source = "«".to_string() + &"(".repeat(600) + "»";
        let result = recursion::check_recursion_depth(&source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recursion_limit_ignored_in_comment() {
        // Parentheses inside comment shouldn't count
        let source = "// ".to_string() + &"(".repeat(600);
        let result = recursion::check_recursion_depth(&source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recursion_limit_mixed_brackets() {
        // Mixed brackets should all count towards the same limit
        // 200 (, 200 {, 101 [ = 501 total
        let source = "(".repeat(200)
            + &"{".repeat(200)
            + &"[".repeat(101)
            + &"]".repeat(101)
            + &"}".repeat(200)
            + &")".repeat(200);
        let result = recursion::check_recursion_depth(&source);
        assert!(matches!(
            result,
            Err(ParseError::RecursionLimitExceeded(500))
        ));
    }

    #[test]
    fn test_recursion_limit_unbalanced_but_safe() {
        // Unbalanced brackets that don't exceed depth
        // (((...))) then (((...))) - sequential, not nested
        let part = "(".repeat(400) + &")".repeat(400);
        let source = part.clone() + &part;
        let result = recursion::check_recursion_depth(&source);
        assert!(result.is_ok());
    }
}
