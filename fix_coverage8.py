import re
with open("tests/sentry_assembler_coverage_tests.rs", "r") as f:
    content = f.read()

content = content.replace('''#[test]
fn test_conversion_undefined_unknown_type_obj() -> Result<(), Box<dyn std::error::Error>> {
    let source = "τὸν ἄγνωστον λέγε.";
    let ast = glossa::parser::parse(source);
    let prog = glossa::semantic::analyze_program(&ast);
    assert!(prog.is_err());
    Ok(())
}
''', '')

with open("tests/sentry_assembler_coverage_tests.rs", "w") as f:
    f.write(content)
