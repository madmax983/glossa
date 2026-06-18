import re

with open('tests/havoc_issue_echo.rs', 'r') as f:
    content = f.read()

# Ah! `let _ = analyze_program(&ast).unwrap();` is BACK?!
# I probably reverted it or lost it in a checkout.
replacement = """#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let err = analyze_program(&ast).unwrap_err();
    assert!(format!("{:?}", err).contains("DoubleSubject"));
}"""

content = re.sub(
    r'#\[test\]\nfn test_double_subject_should_pass_havoc_constraint\(\) \{\n.*?let _ = analyze_program\(&ast\)\.unwrap\(\);\n.*?\}',
    replacement,
    content,
    flags=re.DOTALL
)

with open('tests/havoc_issue_echo.rs', 'w') as f:
    f.write(content)
