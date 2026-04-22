import re

with open("tests/havoc_issue_echo.rs", "r") as f:
    code = f.read()

search_block = """#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let res = analyze_program(&ast);
    assert!(res.is_err());
}"""

replace_block = """#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let res = analyze_program(&ast);
    // Actually wait, this is print verb, so it was bypassing double subject previously. Wait, λέγει is a print verb!
    // Memory says: "the DoubleSubject check should evaluate all verbs uniformly and must not explicitly bypass print verbs (!crate::morphology::lexicon::is_print_verb)"
    // Oh, I only removed `is_print_verb` in `DoubleSubject` when I had an `is_match_arm` change? Let me apply it!
    assert!(res.is_err());
}"""

code = code.replace(search_block, replace_block)

with open("tests/havoc_issue_echo.rs", "w") as f:
    f.write(code)
