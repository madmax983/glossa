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

pub mod numerals;
pub mod recursion;

use crate::ast::*;
use crate::errors::GlossaError;
use crate::grammar::{Rule, parse as grammar_parse};
use pest::iterators::Pair;

impl From<ParseError> for GlossaError {
    fn from(err: ParseError) -> Self {
        GlossaError::parse(err.to_string())
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
    parse_source(source).map_err(GlossaError::from)
}

fn parse_number_literal(text: &str) -> Result<i64, ParseError> {
    if let Ok(val) = text.parse::<i64>() {
        Ok(val)
    } else {
        numerals::parse_greek_numeral(text)
            .map_err(|e| ParseError::InvalidNumber(format!("{} - {}", text, e)))
    }
}

/// Build an AST from source code
fn parse_source(source: &str) -> Result<Program, ParseError> {
    // Check recursion depth before parsing to prevent stack overflow
    recursion::check_recursion_depth(source)?;

    let pairs = grammar_parse(source).map_err(|e| ParseError::PestError(e.to_string()))?;

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

fn build_statement(pair: Pair<'_, Rule>) -> Result<Statement, ParseError> {
    let mut pairs = pair.into_inner();
    let first = pairs
        .next()
        .ok_or(ParseError::UnexpectedRule("Empty statement".into()))?;

    match first.as_rule() {
        Rule::test_declaration => Ok(Statement::TestDeclaration(build_test_declaration(first)?)),
        Rule::type_definition => {
            // Consume statement_end
            let _ = pairs.next();
            Ok(Statement::TypeDefinition(build_type_definition(first)?))
        }
        Rule::trait_definition => {
            // Consume statement_end
            let _ = pairs.next();
            Ok(Statement::TraitDefinition(build_trait_definition(first)?))
        }
        Rule::trait_impl => {
            // Consume statement_end
            let _ = pairs.next();
            Ok(Statement::TraitImpl(build_trait_impl(first)?))
        }
        Rule::clause_list => build_regular_statement(first, pairs),
        _ => Err(ParseError::UnexpectedRule(format!(
            "Unexpected start of statement: {:?}",
            first.as_rule()
        ))),
    }
}

fn build_regular_statement(
    clause_list_pair: Pair<'_, Rule>,
    mut statement_pairs: pest::iterators::Pairs<'_, Rule>,
) -> Result<Statement, ParseError> {
    let clauses = build_clauses(clause_list_pair)?;
    let end_pair = statement_pairs
        .next()
        .ok_or(ParseError::UnexpectedRule("Missing statement end".into()))?;
    let (is_query, is_propagate) = parse_statement_end(end_pair);
    Ok(Statement::Regular {
        clauses,
        is_query,
        is_propagate,
    })
}

fn build_clauses(pair: Pair<'_, Rule>) -> Result<Vec<Clause>, ParseError> {
    let mut clauses = Vec::new();
    for clause_pair in pair.into_inner() {
        if clause_pair.as_rule() == Rule::clause {
            clauses.push(build_clause(clause_pair)?);
        }
    }
    Ok(clauses)
}

fn parse_statement_end(pair: Pair<'_, Rule>) -> (bool, bool) {
    let mut is_query = false;
    let mut is_propagate = false;
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::query => is_query = true,
            Rule::propagate => is_propagate = true,
            _ => {}
        }
    }
    (is_query, is_propagate)
}

fn build_type_definition(pair: Pair<'_, Rule>) -> Result<TypeDef, ParseError> {
    let mut type_name = None;
    let mut fields = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::greek_word => {
                // This is the type name (between εἶδος and ὁρίζειν)
                type_name = Some(Word {
                    original: inner.as_str().into(),
                    normalized: crate::text::normalize_greek(inner.as_str()),
                });
            }
            Rule::field_list => {
                for field_pair in inner.into_inner() {
                    if field_pair.as_rule() == Rule::field_declaration {
                        fields.push(build_field_declaration(field_pair)?);
                    }
                }
            }
            _ => {}
        }
    }

    if type_name.is_none() {
        return Err(ParseError::UnexpectedRule(
            "Type definition needs a name".to_string(),
        ));
    }

    Ok(TypeDef {
        name: type_name.unwrap(),
        fields,
    })
}

fn build_field_declaration(pair: Pair<'_, Rule>) -> Result<FieldDecl, ParseError> {
    let mut words = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::greek_word {
            words.push(Word {
                original: inner.as_str().into(),
                normalized: crate::text::normalize_greek(inner.as_str()),
            });
        }
    }

    // fieldname typename_genitive
    if words.len() != 2 {
        return Err(ParseError::UnexpectedRule(format!(
            "Field declaration needs exactly 2 words, got {}",
            words.len()
        )));
    }

    Ok(FieldDecl {
        name: words[0].clone(),
        type_name: words[1].clone(),
    })
}

fn build_trait_definition(pair: Pair<'_, Rule>) -> Result<TraitDef, ParseError> {
    let mut trait_name = None;
    let mut methods = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::greek_word => {
                // This is the trait name (between χαρακτήρ and ὁρίζειν)
                trait_name = Some(Word {
                    original: inner.as_str().into(),
                    normalized: crate::text::normalize_greek(inner.as_str()),
                });
            }
            Rule::trait_method_list => {
                for method_pair in inner.into_inner() {
                    if method_pair.as_rule() == Rule::trait_method {
                        methods.push(build_trait_method(method_pair)?);
                    }
                }
            }
            _ => {}
        }
    }

    if trait_name.is_none() {
        return Err(ParseError::UnexpectedRule(
            "Trait definition needs a name".to_string(),
        ));
    }

    Ok(TraitDef {
        name: trait_name.unwrap(),
        methods,
    })
}

fn parse_method_parameters(words: &[Word]) -> Vec<FieldDecl> {
    let mut params = Vec::new();
    let mut iter = words.iter();
    while let Some(word) = iter.next() {
        // Look for τῷ (dative marker) followed by parameter name
        if word.normalized != "τω" && word.normalized != "tw" {
            continue;
        }

        if let Some(name) = iter.next() {
            // Parameter without type annotation (just name)
            params.push(FieldDecl {
                name: name.clone(),
                type_name: Word::new("_"), // Placeholder, will be inferred
            });
        }
    }
    params
}

fn build_trait_method(pair: Pair<'_, Rule>) -> Result<TraitMethodDecl, ParseError> {
    let mut words = Vec::new();
    let mut body_statements = Vec::new();
    let mut has_body = false;
    let mut is_default = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::dei_keyword => {
                is_default = false;
            }
            Rule::ede_keyword => {
                is_default = true;
            }
            Rule::greek_word => {
                words.push(Word {
                    original: inner.as_str().into(),
                    normalized: crate::text::normalize_greek(inner.as_str()),
                });
            }
            Rule::statement => {
                has_body = true;
                body_statements.push(build_statement(inner)?);
            }
            _ => {}
        }
    }

    // words[0] = method name
    // words[1..] = parameters (τῷ self, τῷ other, etc.)
    if words.is_empty() {
        return Err(ParseError::UnexpectedRule(
            "Trait method needs at least a name".to_string(),
        ));
    }

    let method_name = words[0].clone();

    // Parse parameters (skip method name)
    let params = parse_method_parameters(&words[1..]);

    Ok(TraitMethodDecl {
        name: method_name,
        params,
        is_default,
        body: if has_body {
            Some(body_statements)
        } else {
            None
        },
    })
}

fn build_trait_impl(pair: Pair<'_, Rule>) -> Result<TraitImplDef, ParseError> {
    let mut type_name = None;
    let mut trait_name = None;
    let mut methods = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::greek_word => {
                // First greek_word is the type name, second is the trait name
                if type_name.is_none() {
                    type_name = Some(Word {
                        original: inner.as_str().into(),
                        normalized: crate::text::normalize_greek(inner.as_str()),
                    });
                } else if trait_name.is_none() {
                    trait_name = Some(Word {
                        original: inner.as_str().into(),
                        normalized: crate::text::normalize_greek(inner.as_str()),
                    });
                }
            }
            Rule::impl_method_list => {
                for method_pair in inner.into_inner() {
                    if method_pair.as_rule() == Rule::impl_method {
                        methods.push(build_impl_method(method_pair)?);
                    }
                }
            }
            _ => {}
        }
    }

    if type_name.is_none() || trait_name.is_none() {
        return Err(ParseError::UnexpectedRule(
            "Trait impl needs type and trait names".to_string(),
        ));
    }

    Ok(TraitImplDef {
        type_name: type_name.unwrap(),
        trait_name: trait_name.unwrap(),
        methods,
    })
}

fn build_test_declaration(pair: Pair<'_, Rule>) -> Result<TestDecl, ParseError> {
    let mut test_name = None;
    let mut body = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::string_literal => {
                // Extract the content from «name»
                for content_pair in inner.into_inner() {
                    if content_pair.as_rule() == Rule::string_content {
                        test_name = Some(content_pair.as_str().to_string());
                    }
                }
            }
            Rule::test_body => {
                // test_body contains statements
                for stmt_pair in inner.into_inner() {
                    if stmt_pair.as_rule() == Rule::statement {
                        body.push(build_statement(stmt_pair)?);
                    }
                }
            }
            Rule::statement => {
                // Fallback for direct statements (shouldn't happen with test_body)
                body.push(build_statement(inner)?);
            }
            Rule::statement_end => {
                // Skip these - they're punctuation
            }
            _ => {
                // Skip keywords and other tokens
            }
        }
    }

    if test_name.is_none() {
        return Err(ParseError::UnexpectedRule(
            "Test declaration needs a name".to_string(),
        ));
    }

    Ok(TestDecl {
        name: test_name.unwrap(),
        body,
    })
}

fn build_impl_method(pair: Pair<'_, Rule>) -> Result<ImplMethodDef, ParseError> {
    let mut words = Vec::new();
    let mut body = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::greek_word => {
                words.push(Word {
                    original: inner.as_str().into(),
                    normalized: crate::text::normalize_greek(inner.as_str()),
                });
            }
            Rule::statement => {
                // Each impl_method has exactly one statement (use block for multiple)
                body = Some(build_statement(inner)?);
            }
            _ => {}
        }
    }

    // words[0] = method name
    // words[1..] = parameters (τῷ self, τῷ other, etc.)
    if words.is_empty() {
        return Err(ParseError::UnexpectedRule(
            "Impl method needs at least a name".to_string(),
        ));
    }

    let method_name = words[0].clone();

    // Parse parameters (skip method name)
    let params = parse_method_parameters(&words[1..]);

    Ok(ImplMethodDef {
        name: method_name,
        params,
        body: if let Some(stmt) = body {
            vec![stmt]
        } else {
            vec![]
        },
    })
}

fn build_clause(pair: Pair<'_, Rule>) -> Result<Clause, ParseError> {
    let mut expressions = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expression {
            expressions.push(build_expression(inner)?);
        }
    }
    Ok(Clause { expressions })
}

fn build_expression(pair: Pair<'_, Rule>) -> Result<Expr, ParseError> {
    let mut terms = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::term {
            terms.push(build_term(inner)?);
        }
    }

    // If there's only one term, return it directly
    if terms.len() == 1 {
        Ok(terms.into_iter().next().unwrap())
    } else {
        // Multiple terms form a phrase (e.g., "χαῖρε κόσμε λέγε")
        Ok(Expr::Phrase(terms))
    }
}

fn build_block_expr(inner: Pair<'_, Rule>) -> Result<Expr, ParseError> {
    let mut statements = Vec::new();
    for stmt_pair in inner.into_inner() {
        if stmt_pair.as_rule() == Rule::statement {
            statements.push(build_statement(stmt_pair)?);
        }
    }
    Ok(Expr::Block(statements))
}

fn build_array_literal_expr(inner: Pair<'_, Rule>) -> Result<Expr, ParseError> {
    let mut elements = Vec::new();
    for child in inner.into_inner() {
        if child.as_rule() == Rule::array_elements {
            for elem in child.into_inner() {
                if elem.as_rule() == Rule::array_element {
                    elements.push(build_array_element(elem)?);
                }
            }
        }
    }
    Ok(Expr::ArrayLiteral(elements))
}

fn build_indexed_word_expr(inner: Pair<'_, Rule>) -> Result<Expr, ParseError> {
    let mut parts = inner.into_inner();
    // First is the greek_word (array name)
    let array_word = parts.next().ok_or(ParseError::EmptyTerm)?;
    let array = Expr::Word(Word {
        original: array_word.as_str().into(),
        normalized: crate::text::normalize_greek(array_word.as_str()),
    });
    // Second is the index_expr
    let index_pair = parts.next().ok_or(ParseError::EmptyTerm)?;
    let index_inner = index_pair
        .into_inner()
        .next()
        .ok_or(ParseError::EmptyTerm)?;
    let index = match index_inner.as_rule() {
        Rule::number_literal => {
            let value = parse_number_literal(index_inner.as_str())?;
            Expr::NumberLiteral(value)
        }
        Rule::greek_word => Expr::Word(Word {
            original: index_inner.as_str().into(),
            normalized: crate::text::normalize_greek(index_inner.as_str()),
        }),
        _ => {
            return Err(ParseError::UnexpectedRule(format!(
                "{:?}",
                index_inner.as_rule()
            )));
        }
    };
    Ok(Expr::IndexAccess {
        array: Box::new(array),
        index: Box::new(index),
    })
}

fn build_unwrap_expr(inner: Pair<'_, Rule>) -> Result<Expr, ParseError> {
    // Extract the word from "word!"
    let word_pair = inner.into_inner().next().ok_or(ParseError::EmptyTerm)?;
    let word = Expr::Word(Word {
        original: word_pair.as_str().into(),
        normalized: crate::text::normalize_greek(word_pair.as_str()),
    });
    Ok(Expr::UnaryOp {
        op: UnaryOperator::Unwrap,
        operand: Box::new(word),
    })
}

fn build_term(pair: Pair<'_, Rule>) -> Result<Expr, ParseError> {
    let inner = pair.into_inner().next().ok_or(ParseError::EmptyTerm)?;

    match inner.as_rule() {
        Rule::block => build_block_expr(inner),
        Rule::array_literal => build_array_literal_expr(inner),
        Rule::indexed_word => build_indexed_word_expr(inner),
        Rule::parenthesized_expr => {
            // Unwrap the parentheses and build the inner expression
            let expr_pair = inner.into_inner().next().ok_or(ParseError::EmptyTerm)?;
            build_expression(expr_pair)
        }
        Rule::unwrap_expr => build_unwrap_expr(inner),
        // Literals
        Rule::string_literal | Rule::number_literal | Rule::boolean_literal | Rule::greek_word => {
            build_literal(inner)
        }
        _ => Err(ParseError::UnexpectedRule(format!("{:?}", inner.as_rule()))),
    }
}

fn build_array_element(pair: Pair<'_, Rule>) -> Result<Expr, ParseError> {
    let inner = pair.into_inner().next().ok_or(ParseError::EmptyTerm)?;
    build_literal(inner)
}

fn build_literal(pair: Pair<'_, Rule>) -> Result<Expr, ParseError> {
    match pair.as_rule() {
        Rule::string_literal => {
            let content = pair
                .into_inner()
                .find(|p| p.as_rule() == Rule::string_content)
                .map(|p| p.as_str().to_string())
                .unwrap_or_default();
            Ok(Expr::StringLiteral(content))
        }
        Rule::number_literal => {
            let value = parse_number_literal(pair.as_str())?;
            Ok(Expr::NumberLiteral(value))
        }
        Rule::boolean_literal => {
            let normalized = crate::text::normalize_greek(pair.as_str());
            let value = normalized == "αληθες";
            Ok(Expr::BooleanLiteral(value))
        }
        Rule::greek_word => Ok(Expr::Word(Word {
            original: pair.as_str().into(),
            normalized: crate::text::normalize_greek(pair.as_str()),
        })),
        _ => Err(ParseError::UnexpectedRule(format!(
            "Not a literal: {:?}",
            pair.as_rule()
        ))),
    }
}

/// Errors that can occur during AST construction
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    PestError(String),

    #[error("Empty term in expression")]
    EmptyTerm,

    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    #[error("Unexpected rule: {0}")]
    UnexpectedRule(String),

    #[error("Recursion limit exceeded: depth > {0}")]
    RecursionLimitExceeded(usize),
}

#[cfg(test)]
mod tests {
    use super::recursion::check_recursion_depth;
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
        let result = check_recursion_depth(&source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recursion_limit_ignored_in_string() {
        // Parentheses inside string literal shouldn't count
        let source = "«".to_string() + &"(".repeat(600) + "»";
        let result = check_recursion_depth(&source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recursion_limit_ignored_in_comment() {
        // Parentheses inside comment shouldn't count
        let source = "// ".to_string() + &"(".repeat(600);
        let result = check_recursion_depth(&source);
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
        let result = check_recursion_depth(&source);
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
        let result = check_recursion_depth(&source);
        assert!(result.is_ok());
    }
}
