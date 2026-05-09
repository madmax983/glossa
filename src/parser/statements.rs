//! Statement Parsing
//!
//! Handles parsing of grammatical clauses into regular statements.
use crate::ast::*;
use crate::parser::common::ParseError;
use crate::parser::core::Rule;
use crate::parser::expressions::build_expression;
use pest::iterators::Pair;

pub(crate) fn build_regular_statement(
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

/// Extracts clauses from a grammar statement pair.
///
/// ⚡ Bolt Optimization: Uses `Vec::with_capacity(inner.len())`
/// This reduces heap reallocations when parsing statements containing multiple clauses.
fn build_clauses(pair: Pair<'_, Rule>) -> Result<Vec<Clause>, ParseError> {
    let inner = pair.into_inner();
    let mut clauses = Vec::with_capacity(inner.len());
    for clause_pair in inner {
        if clause_pair.as_rule() == Rule::clause {
            clauses.push(build_clause(clause_pair)?);
        }
    }
    Ok(clauses)
}

/// Extracts expressions from a single clause pair.
///
/// ⚡ Bolt Optimization: Uses `Vec::with_capacity(inner.len())`
/// Limits dynamic memory allocations on the most critical AST generation paths.
fn build_clause(pair: Pair<'_, Rule>) -> Result<Clause, ParseError> {
    let inner = pair.into_inner();
    let mut expressions = Vec::with_capacity(inner.len());

    for inner_pair in inner {
        if inner_pair.as_rule() == Rule::expression {
            expressions.push(build_expression(inner_pair)?);
        }
    }
    Ok(Clause { expressions })
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

#[cfg(test)]
mod tests_coverage {
    use super::*;
    use crate::parser::core::{GlossaParser, Rule};
    use pest::Parser;

    #[test]
    fn test_build_regular_statement_missing_end() {
        let mut pairs = GlossaParser::parse(Rule::clause_list, "ξ λέγε").unwrap();
        let clause_list_pair = pairs.next().unwrap();

        let empty_pairs = GlossaParser::parse(Rule::block, "{}").unwrap();
        let inner_pairs = empty_pairs.into_iter().next().unwrap().into_inner();

        let result = build_regular_statement(clause_list_pair, inner_pairs);
        assert!(matches!(result, Err(ParseError::UnexpectedRule(_))));
    }
}
