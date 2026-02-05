use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_diacritic_only_variable_panic() {
    // A Greek word consisting ONLY of a combining diacritic (e.g. U+0301 combining acute accent)
    // The grammar allows GREEK_CHAR+, where GREEK_CHAR includes GREEK_COMBINING.
    // So "\u{0301}" is a valid greek_word.
    // Normalized form is empty string "".

    // Pattern: [var] νέον [Type] ἔστω.
    // "\u{0301} νέον Σύνολον ἔστω." -> Let '´' be a new Set.

    let source = "\u{0301} νέον Σύνολον ἔστω.";

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    // This expects to NOT panic inside generate_rust
    let rust_code = generate_rust(&analyzed);

    // Verify that it generated a safe fallback name
    assert!(
        rust_code.contains("_var_empty"),
        "Generated code should contain fallback for empty identifier"
    );
}

#[test]
fn test_stack_overflow_nested_parens() {
    // 👺 Havoc: Deep Recursion Attack
    // Create a deeply nested expression: (((((...)))))
    let depth = 50000;
    let mut source = String::with_capacity(depth * 2 + 10);
    for _ in 0..depth {
        source.push('(');
    }
    source.push('1'); // The core
    for _ in 0..depth {
        source.push(')');
    }
    source.push('.'); // End statement

    // This should NOT crash with a stack overflow, but return an error
    let result = parse(&source);
    assert!(result.is_err(), "Deep recursion should fail gracefully");

    // Optional: verify error message contains recursion limit
    // but GlossaError wraps ParseError, so checking string representation is easiest
    let err_msg = result.err().unwrap().to_string();
    assert!(
        err_msg.contains("Recursion limit exceeded") || err_msg.contains("Parse error"),
        "Unexpected error: {}",
        err_msg
    );
}
