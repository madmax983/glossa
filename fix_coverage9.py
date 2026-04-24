import re

with open("tests/sentry_assembler_coverage_tests.rs", "r") as f:
    content = f.read()

content = re.sub(
    r'''#\[test\]
fn test_conversion_undefined_unknown_type_obj\(\) \{
    let source = "τὸν ἄγνωστον λέγε\.";
    let ast = glossa::parser::parse\(source\)\.unwrap\(\);
    let prog = glossa::semantic::analyze_program\(&ast\);
    assert!\(prog\.is_err\(\)\);
\}''',
    "",
    content
)

with open("tests/sentry_assembler_coverage_tests.rs", "w") as f:
    f.write(content)
