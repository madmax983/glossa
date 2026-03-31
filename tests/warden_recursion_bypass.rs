#![allow(missing_docs)]
use glossa::parser::parse;

#[test]
fn test_exploit_recursion_bypass() {
    let mut s = String::new();
    let depth = 600; // This should trigger stack overflow if limit is bypassed

    // Nest using unaccented "δοκιμη"
    for _ in 0..depth {
        s.push_str("δοκιμη «test». ");
    }

    s.push_str("1 λέγε. ");

    // End using unaccented "τελος"
    for _ in 0..depth {
        s.push_str("τελος. ");
    }

    let result = parse(&s);
    // It should fail with RecursionLimitExceeded, not crash with stack overflow
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("Recursion limit exceeded"),
        "Error was: {}",
        err_str
    );
}
