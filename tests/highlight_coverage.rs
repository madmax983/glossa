use glossa::highlight::highlight;

#[test]
fn test_highlight_integration() {
    let source = "«χαῖρε» λέγε.";
    let res = highlight(source);
    assert!(res.is_ok());
    let output = res.unwrap();
    // Verify it contains ANSI codes (which highlight produces)
    // and the original text
    assert!(output.contains("χαῖρε"));
    assert!(output.contains("\x1b["));
}

#[test]
fn test_highlight_complex() {
    let source = "ξ πέντε ἔστω. ξ λέγε.";
    let res = highlight(source);
    assert!(res.is_ok());
    let output = res.unwrap();
    assert!(output.contains("ξ"));
    assert!(output.contains("πέντε"));
}
