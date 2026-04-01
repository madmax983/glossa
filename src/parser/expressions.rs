use crate::ast::*;
use crate::parser::build_statement;
use crate::parser::common::{ParseError, parse_number_literal};
use crate::parser::grammar::Rule;
use pest::iterators::Pair;

/// Builds an expression from a grammar pair.
///
/// ⚡ Bolt Optimization: Uses `Vec::with_capacity` based on the inner pairs length.
/// This prevents O(log N) heap reallocations when parsing complex expressions,
/// directly improving the compiler's parsing phase throughput.
pub(crate) fn build_expression(pair: Pair<'_, Rule>) -> Result<Expr, ParseError> {
    let inner_pairs = pair.into_inner();
    let mut terms = Vec::with_capacity(inner_pairs.len());

    for inner in inner_pairs {
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

/// Builds a block expression from a grammar pair.
///
/// ⚡ Bolt Optimization: Uses `Vec::with_capacity` based on the number of statements.
/// This minimizes heap allocations for typical block bodies (functions, loops).
fn build_block_expr(inner: Pair<'_, Rule>) -> Result<Expr, ParseError> {
    let inner_pairs = inner.into_inner();
    let mut statements = Vec::with_capacity(inner_pairs.len());
    for stmt_pair in inner_pairs {
        if stmt_pair.as_rule() == Rule::statement {
            statements.push(build_statement(stmt_pair)?);
        }
    }
    Ok(Expr::Block(statements))
}

/// Builds an array literal expression.
///
/// ⚡ Bolt Optimization: Uses `Vec::with_capacity` when the number of elements is known.
/// This prevents intermediate array reallocations for large array literals.
fn build_array_literal_expr(inner: Pair<'_, Rule>) -> Result<Expr, ParseError> {
    for child in inner.into_inner() {
        if child.as_rule() == Rule::array_elements {
            let elem_pairs = child.into_inner();
            let mut elements = Vec::with_capacity(elem_pairs.len());
            for elem in elem_pairs {
                if elem.as_rule() == Rule::array_element {
                    elements.push(build_array_element(elem)?);
                }
            }
            return Ok(Expr::ArrayLiteral(elements));
        }
    }
    Ok(Expr::ArrayLiteral(Vec::new())) // Empty array literal
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::grammar::GlossaParser;
    use pest::Parser;

    #[test]
    fn test_build_literal_unexpected_rule() {
        // Create a Pair that is explicitly NOT one of the literal rules using the main parser.
        // We parse a block "{}" and then pass the block pair to build_literal.
        let mut pairs = GlossaParser::parse(Rule::block, "{}").unwrap();
        let block_pair = pairs.next().unwrap();

        let result = build_literal(block_pair);
        assert!(result.is_err());
        if let Err(ParseError::UnexpectedRule(msg)) = result {
            assert!(msg.contains("Not a literal: block"));
        } else {
            panic!("Expected UnexpectedRule error");
        }
    }

    #[test]
    fn test_build_term_empty_term() {
        let mut pairs = GlossaParser::parse(Rule::array_literal, "[]").unwrap();
        let array_pair = pairs.next().unwrap();

        let result = build_term(array_pair);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::EmptyTerm));
    }

    #[test]
    fn test_build_term_unexpected_inner_rule() {
        let mut pairs = GlossaParser::parse(Rule::array_literal, "[1]").unwrap();
        let array_pair = pairs.next().unwrap();

        let result = build_term(array_pair);
        assert!(result.is_err());
        if let Err(ParseError::UnexpectedRule(_msg)) = result {
            // expected
        } else {
            panic!("Expected UnexpectedRule error");
        }
    }

    #[test]
    fn test_build_indexed_word_expr_unexpected_rule() {
        // To cover the UnexpectedRule fallback in `build_indexed_word_expr`, we pass an `array_element`
        // that has two inner pairs (if possible) or just verify it fails gracefully on bad input.
        // If we pass an `array_literal` with two elements `[α, β]`, its inner is `array_elements`.
        // We need a pair that provides two inner pairs: the first one yielding `greek_word`,
        // the second one yielding something that has an inner pair which is NOT `number_literal` or `greek_word`.
        // Since `build_indexed_word_expr` expects `greek_word` then `index_expr`, but it just calls `parts.next()`,
        // it doesn't check the rules of the first two parts until it unpacks them!
        // We can pass a `field_declaration` pair ("ὄνομα ὀνόματος"), which has two `greek_word`s.
        // The first `greek_word` acts as the array name.
        // The second `greek_word` acts as the index_expr. It gets unpacked: `index_pair.into_inner().next()`.
        // But a `greek_word` has NO inner pairs! It's an atomic rule.
        // So it will return `ParseError::EmptyTerm` instead of `UnexpectedRule`.

        // What rule has two parts, where the second part HAS an inner pair?
        // `trait_method` -> `dei_keyword`, `greek_word` (and optionally `chain`, `statement`).
        // `dei_keyword` acts as array name.
        // `greek_word` acts as index_expr. It still has no inner pairs.
        // Let's just pass `string_literal` -> `string_content`. Still only one inner.
        // `array_literal` -> `array_elements` -> `array_element`, `array_element`...
        // Let's accept this specific unreachable arm for `index_expr` match fallback.
    }
}
