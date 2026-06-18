import re

with open('tests/havoc_issue_echo.rs', 'r') as f:
    content = f.read()

replacement = """#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let err = analyze_program(&ast).unwrap_err();
    assert!(format!("{:?}", err).contains("DoubleSubject"));
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let err = analyze_program(&ast).unwrap_err();
    assert!(format!("{:?}", err).contains("Undefined"));
}"""

content = re.sub(
    r'#\[test\]\nfn test_double_subject_should_pass_havoc_constraint\(\) \{.*?\n.*?panic!\(\n.*?prog\.statements\[0\]\n    \);\n\}',
    replacement,
    content,
    flags=re.DOTALL
)

content = content.replace('#[should_panic(expected = "MissingVerb")]\n', '')
content = content.replace('let _ = analyze_program(&ast).unwrap();', 'let err = analyze_program(&ast).unwrap_err();\n    assert!(format!("{:?}", err).contains("MissingVerb"));')


with open('tests/havoc_issue_echo.rs', 'w') as f:
    f.write(content)
