use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_dos_max_tokens_bypass() {
    // Generate a huge statement with nested phrases
    // (a b) (a b) ... repeated 2000 times
    // This creates 2000 nested phrases.
    // MAX_TOKENS is 1000.

    let mut source = String::new();
    for _ in 0..2000 {
        source.push_str("(α β) ");
    }
    source.push_str("λέγε."); // End with a verb to make it a valid statement

    // Parse
    let ast = parse(&source).expect("Parsing should succeed");

    // Analyze - this should fail with StatementTooLong
    let result = analyze_program(&ast);

    if result.is_ok() {
        panic!("DoS EXPLOIT SUCCESSFUL: Managed to feed 2000+ tokens bypassing MAX_TOKENS limit!");
    }

    let err = result.err().unwrap();
    println!("Got error: {}", err);

    // Verify it is indeed the token limit error
    // Error message: "Πρότασις λίαν μακρά! ({count} > {limit})"
    assert!(
        err.to_string().contains("Πρότασις λίαν μακρά"),
        "Error should be 'Statement too long' (in Greek), got: {}",
        err
    );
}
