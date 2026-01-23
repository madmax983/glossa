//! Abstract Syntax Tree for ΓΛΩΣΣΑ
//!
//! The AST captures the semantic structure of a GLOSSA program,
//! preserving morphological information from Greek words.

mod nodes;

pub use nodes::*;

use crate::grammar::{Rule, parse};
use pest::iterators::Pair;

/// Build an AST from source code
pub fn build_ast(source: &str) -> Result<Program, AstError> {
    let pairs = parse(source).map_err(|e| AstError::ParseError(e.to_string()))?;

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

fn build_statement(pair: Pair<'_, Rule>) -> Result<Statement, AstError> {
    let mut expressions = Vec::new();
    let mut is_query = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expression_list => {
                for expr_pair in inner.into_inner() {
                    if expr_pair.as_rule() == Rule::expression {
                        expressions.push(build_expression(expr_pair)?);
                    }
                }
            }
            Rule::statement_end => {
                for end_inner in inner.into_inner() {
                    if end_inner.as_rule() == Rule::query {
                        is_query = true;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Statement {
        expressions,
        is_query,
    })
}

fn build_expression(pair: Pair<'_, Rule>) -> Result<Expr, AstError> {
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

fn build_term(pair: Pair<'_, Rule>) -> Result<Expr, AstError> {
    let inner = pair.into_inner().next().ok_or(AstError::EmptyTerm)?;

    match inner.as_rule() {
        Rule::string_literal => {
            let content = inner.into_inner()
                .find(|p| p.as_rule() == Rule::string_content)
                .map(|p| p.as_str().to_string())
                .unwrap_or_default();
            Ok(Expr::StringLiteral(content))
        }
        Rule::number_literal => {
            let value: i64 = inner.as_str().parse()
                .map_err(|_| AstError::InvalidNumber(inner.as_str().to_string()))?;
            Ok(Expr::NumberLiteral(value))
        }
        Rule::boolean_literal => {
            let normalized = crate::grammar::normalize_greek(inner.as_str());
            let value = normalized == "αληθες";
            Ok(Expr::BooleanLiteral(value))
        }
        Rule::greek_word => {
            Ok(Expr::Word(Word {
                original: inner.as_str().to_string(),
                normalized: crate::grammar::normalize_greek(inner.as_str()),
            }))
        }
        _ => Err(AstError::UnexpectedRule(format!("{:?}", inner.as_rule()))),
    }
}

/// Errors that can occur during AST construction
#[derive(Debug, Clone, thiserror::Error)]
pub enum AstError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Empty term in expression")]
    EmptyTerm,

    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    #[error("Unexpected rule: {0}")]
    UnexpectedRule(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ast_hello() {
        let source = "«χαῖρε» λέγε.";
        let ast = build_ast(source).unwrap();

        assert_eq!(ast.statements.len(), 1);
        assert!(!ast.statements[0].is_query);
    }

    #[test]
    fn test_build_ast_string_literal() {
        let source = "«χαῖρε κόσμε» λέγε.";
        let ast = build_ast(source).unwrap();

        let expr = &ast.statements[0].expressions[0];
        if let Expr::Phrase(terms) = expr {
            assert!(matches!(&terms[0], Expr::StringLiteral(s) if s == "χαῖρε κόσμε"));
        } else {
            panic!("Expected Phrase, got {:?}", expr);
        }
    }

    #[test]
    fn test_build_ast_variable_binding() {
        let source = "ξ πέντε ἔστω.";
        let ast = build_ast(source).unwrap();

        assert_eq!(ast.statements.len(), 1);
        let expr = &ast.statements[0].expressions[0];

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
    fn test_build_ast_number_literal() {
        let source = "42 λέγε.";
        let ast = build_ast(source).unwrap();

        let expr = &ast.statements[0].expressions[0];
        if let Expr::Phrase(terms) = expr {
            assert!(matches!(&terms[0], Expr::NumberLiteral(42)));
        }
    }

    #[test]
    fn test_build_ast_query() {
        let source = "ξ?";
        let ast = build_ast(source).unwrap();

        assert!(ast.statements[0].is_query);
    }

    #[test]
    fn test_build_ast_multiple_statements() {
        let source = "ξ πέντε ἔστω. ξ λέγε.";
        let ast = build_ast(source).unwrap();

        assert_eq!(ast.statements.len(), 2);
    }

    #[test]
    fn test_word_normalization() {
        let source = "χρήστου ὄνομα λέγε.";
        let ast = build_ast(source).unwrap();

        let expr = &ast.statements[0].expressions[0];
        if let Expr::Phrase(terms) = expr {
            if let Expr::Word(w) = &terms[0] {
                assert_eq!(w.original, "χρήστου");
                assert_eq!(w.normalized, "χρηστου");
            }
        }
    }
}
