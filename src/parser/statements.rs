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

fn build_clauses(pair: Pair<'_, Rule>) -> Result<Vec<Clause>, ParseError> {
    let mut clauses = Vec::new();
    for clause_pair in pair.into_inner() {
        if clause_pair.as_rule() == Rule::clause {
            clauses.push(build_clause(clause_pair)?);
        }
    }
    Ok(clauses)
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
