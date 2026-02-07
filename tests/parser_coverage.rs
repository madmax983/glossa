use glossa::parser::parse;
use miette::Diagnostic;

#[test]
fn test_unexpected_token_has_labels() {
    let source = "«χαῖρε» @";
    let err = parse(source).unwrap_err();

    // Check that we got the span correctly
    assert!(err.labels().is_some());
    let labels: Vec<_> = err.labels().unwrap().collect();
    assert!(!labels.is_empty());

    // Check that we got an expected message
    // "Expected one of: ..."
    let msg = err.to_string();
    // In miette, `to_string` on the error might not include the full report if it just uses Display,
    // but GlossaError uses `#[error("Σφάλμα συντάξεως: {message}")]` so it should be visible.
    // However, the `ParseError` conversion logic I added constructs a message that says "Expected one of: ..."
    // Let's verify that message is present.
    // Wait, `GlossaError::parse_with_source` takes a message.
    // The message is "Expected one of: ...".
    // So `err.to_string()` will be "Σφάλμα συντάξεως: Expected one of: ..."
    assert!(msg.contains("Expected one of:"));
}

#[test]
fn test_unexpected_eof() {
    let source = "«χαῖρε"; // Unclosed string
    let err = parse(source).unwrap_err();

    assert!(err.labels().is_some());
}
