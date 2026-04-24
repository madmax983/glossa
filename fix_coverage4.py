import re

with open("tests/sentry_assembler_coverage_tests.rs", "r") as f:
    content = f.read()

content += '''
#[test]
fn test_assembler_coverage_missing_verb_complex_expr() {
    let mut asm = glossa::semantic::assembly::Assembler::new();
    let subj = glossa::morphology::analyze_all("ανθρωπος").unwrap();
    asm.feed(&subj[0], "ἄνθρωπος").unwrap();
    let obj = glossa::morphology::analyze_all("λογον").unwrap();
    asm.feed(&obj[0], "λόγον").unwrap();
    let res = asm.finalize();
    assert!(res.is_err());
}

#[test]
fn test_assembler_coverage_missing_verb_complex_expr_obj() {
    let mut asm = glossa::semantic::assembly::Assembler::new();
    let obj = glossa::morphology::analyze_all("λογον").unwrap();
    asm.feed(&obj[0], "λόγον").unwrap();
    let subj = glossa::morphology::analyze_all("ανθρωπος").unwrap();
    asm.feed(&subj[0], "ἄνθρωπος").unwrap();
    let res = asm.finalize();
    assert!(res.is_err());
}

#[test]
fn test_conversion_undefined_unknown_type() {
    let source = "ἄγνωστος λέγε.";
    let ast = glossa::parser::parse(source).unwrap();
    let prog = glossa::semantic::analyze_program(&ast);
    assert!(prog.is_err());
}

#[test]
fn test_conversion_undefined_unknown_type_obj() {
    let source = "τὸν ἄγνωστον λέγε.";
    let ast = glossa::parser::parse(source).unwrap();
    let prog = glossa::semantic::analyze_program(&ast);
    assert!(prog.is_err());
}
'''

with open("tests/sentry_assembler_coverage_tests.rs", "w") as f:
    f.write(content)
