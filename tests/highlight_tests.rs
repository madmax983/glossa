use glossa::highlight::highlight;

#[test]
fn test_highlight_function_call_coverage() {
    let source = "«χαῖρε» λέγε.";
    let res = highlight(source);
    assert!(res.is_ok());
    let output = res.unwrap();
    assert!(output.contains("\x1b[")); // Contains ANSI codes
    assert!(output.contains("χαῖρε"));
    assert!(output.contains("λέγε"));
}

#[test]
fn test_highlight_complex_struct_coverage() {
    let source = "εἶδος Χρήστης ὁρίζειν { ὄνομα Ὄνομα . ἡλικία Ἀριθμός }.";
    let res = highlight(source);
    assert!(res.is_ok());
    let output = res.unwrap();
    assert!(output.contains("Χρήστης"));
}
