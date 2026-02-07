use glossa::parser::parse;
use miette::Diagnostic;

#[test]
fn test_parse_error_span() {
    let source = "«χαῖρε» @ λέγε.";
    let err = parse(source).unwrap_err();

    // Check it's a parse error
    assert_eq!(err.category_greek(), "Σύνταξις");

    // Check labels (spans)
    // labels() returns Option<Box<dyn Iterator...>>
    let labels: Vec<_> = err.labels().expect("Should have labels").collect();
    assert_eq!(labels.len(), 1, "Should have exactly one label");

    let label = &labels[0];
    let offset = label.offset();
    let len = label.len();

    // "«χαῖρε» " is 15 + 1 = 16 bytes.
    // @ is at byte 16.
    assert_eq!(offset, 16, "Offset should point to @");
    assert_eq!(len, 1, "Length should be 1 (@)");
}
