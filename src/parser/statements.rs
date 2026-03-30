//! The Logic Builder (Ὁ Δομήτωρ Λόγου)
//!
//! This module acts as the "Builder" (Δομήτωρ) of the abstract syntax tree's logical spine.
//! While `expressions.rs` focuses on extracting values and terms, this module
//! focuses on assembling those terms into complete sentences (statements) that command
//! the program's execution flow.
//!
//! # Why it exists
//!
//! In ΓΛΩΣΣΑ, statements are not just lines of code separated by semicolons; they are
//! grammatically structured Greek sentences ending in periods (`.`), question marks (`?`),
//! or propagation markers (`:`). This module exists to convert the generic nested structures
//! produced by the Pest parser (`pest::iterators::Pair`) into strongly-typed `Statement` enums
//! (like `Regular`, `If`, `While`, `For`, etc.), preserving the intricate clause structure
//! and identifying control-flow tokens like `εἰ` (if) or `ἕως` (while).
//!
//! # Note
//!
//! These functions are primarily internal to the `parser` crate, invoked by [`crate::parser::parse`].

use crate::ast::*;
use crate::parser::common::ParseError;
use crate::parser::expressions::build_expression;
use crate::parser::grammar::Rule;
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
