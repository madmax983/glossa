#![allow(missing_docs)]
use glossa::parser::parse;

#[test]
fn havoc_does_not_crash_on_extreme_block_nesting() {
    let mut s = String::new();
    let depth = 600; // This used to crash with MAX_PARSE_DEPTH = 500
    for _ in 0..depth {
        s.push_str("{ ");
    }
    s.push_str("1 λέγε.");
    for _ in 0..depth {
        s.push_str(" }");
    }

    let result = parse(&s);
    assert!(result.is_err());

    // Check that it's specifically a RecursionLimitExceeded error
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Recursion limit exceeded"));
}

#[test]
fn havoc_does_not_crash_on_extreme_paren_nesting() {
    let mut s = String::new();
    let depth = 600;
    for _ in 0..depth {
        s.push('(');
    }
    s.push('1');
    for _ in 0..depth {
        s.push(')');
    }
    s.push_str(" λέγε.");

    let result = parse(&s);
    assert!(result.is_err());

    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Recursion limit exceeded"));
}
