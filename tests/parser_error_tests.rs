#![allow(missing_docs)]
use glossa::parser::parse;

#[cfg(test)]
mod tests {
    use super::*;

    // Since `parse` goes through the PEG grammar, the `type_definition` rule
    // explicitly requires `eidos_keyword ~ greek_word`. If `greek_word` is missing,
    // the parser falls back to the `clause_list` rule.
    // So "εἶδος ὁρίζειν { }." becomes a `Regular` statement with three terms.
    // Therefore, the `is_none()` branches we changed are essentially unreachable
    // from valid strings, but replacing `unwrap()` prevents potential panics if
    // the grammar is manually constructed or relaxed in the future.

    // We can at least test that `parse` rejects truly invalid tokens completely.
    #[test]
    fn test_empty_term_handling() {
        let result = parse("().");
        assert!(result.is_err());
    }

    // Since we cannot construct `Pair` manually to trigger those specific errors,
    // we document that these branches are unreachable via string parsing but act as safety guards.
}
