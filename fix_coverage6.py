import re

with open("tests/sentry_assembler_coverage_tests.rs", "r") as f:
    content = f.read()

content = content.replace('.unwrap_or_default();', '?;')

content = re.sub(
    r'''fn test_assembler_coverage_missing_verb_complex_expr\(\) \{
    let mut asm = glossa::semantic::assembly::Assembler::new\(\);
    let subj = glossa::morphology::analyze_all\("ανθρωπος"\)\?;
    let _ = asm\.feed\(&subj\[0\], "ἄνθρωπος"\);
    let obj = glossa::morphology::analyze_all\("λογον"\)\?;
    let _ = asm\.feed\(&obj\[0\], "λόγον"\);
    let res = asm\.finalize\(\);
    assert!\(res\.is_err\(\)\);
\}''',
    r'''fn test_assembler_coverage_missing_verb_complex_expr() -> Result<(), glossa::errors::GlossaError> {
    let mut asm = glossa::semantic::assembly::Assembler::new();
    let subj = glossa::morphology::analyze_all("ανθρωπος")?;
    let _ = asm.feed(&subj[0], "ἄνθρωπος");
    let obj = glossa::morphology::analyze_all("λογον")?;
    let _ = asm.feed(&obj[0], "λόγον");
    let res = asm.finalize();
    assert!(res.is_err());
    Ok(())
}''',
    content
)

content = re.sub(
    r'''fn test_assembler_coverage_missing_verb_complex_expr_obj\(\) \{
    let mut asm = glossa::semantic::assembly::Assembler::new\(\);
    let obj = glossa::morphology::analyze_all\("λογον"\)\?;
    let _ = asm\.feed\(&obj\[0\], "λόγον"\);
    let subj = glossa::morphology::analyze_all\("ανθρωπος"\)\?;
    let _ = asm\.feed\(&subj\[0\], "ἄνθρωπος"\);
    let res = asm\.finalize\(\);
    assert!\(res\.is_err\(\)\);
\}''',
    r'''fn test_assembler_coverage_missing_verb_complex_expr_obj() -> Result<(), glossa::errors::GlossaError> {
    let mut asm = glossa::semantic::assembly::Assembler::new();
    let obj = glossa::morphology::analyze_all("λογον")?;
    let _ = asm.feed(&obj[0], "λόγον");
    let subj = glossa::morphology::analyze_all("ανθρωπος")?;
    let _ = asm.feed(&subj[0], "ἄνθρωπος");
    let res = asm.finalize();
    assert!(res.is_err());
    Ok(())
}''',
    content
)

with open("tests/sentry_assembler_coverage_tests.rs", "w") as f:
    f.write(content)
