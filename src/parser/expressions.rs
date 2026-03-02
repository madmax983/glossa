use crate::ast::*;
use crate::parser::build_statement;
use crate::parser::common::{ParseError, parse_number_literal};
use crate::parser::grammar::Rule;
use pest::iterators::Pair;

pub(crate) fn build_expression(pair: Pair<'_, Rule>) -> Result<Expr, ParseError> {
    let mut terms = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::term {
            terms.push(build_term(inner)?);
        }
    }

    // If there's only one term, return it directly
    if terms.len() == 1 {
        Ok(terms.into_iter().next().ok_or(ParseError::EmptyTerm)?)
    } else {
        // Multiple terms form a phrase (e.g., "χαῖρε κόσμε λέγε")
        Ok(Expr::Phrase(terms)) // Empty terms vector returns an empty Phrase
    }
}

pub(crate) fn build_term(pair: Pair<'_, Rule>) -> Result<Expr, ParseError> {
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
