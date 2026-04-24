#![allow(missing_docs)]
use glossa::morphology::BinaryOp;
use glossa::morphology::analyze;
use glossa::semantic::Assembler;
use glossa::semantic::Literal;

#[test]
fn test_assembler_comparison_operator() {
    let mut asm = Assembler::new();
    let analysis = analyze("ἴσον"); // "equal to"
    asm.feed(&analysis, "ἴσον").unwrap();
    let final_stmt = asm.finalize().unwrap();
    assert_eq!(final_stmt.operators, vec![BinaryOp::Eq]);
}

#[test]
fn test_assembler_arithmetic_operator() {
    let mut asm = Assembler::new();
    let analysis = analyze("ἄθροισμα"); // "sum"
    asm.feed(&analysis, "ἄθροισμα").unwrap();
    let final_stmt = asm.finalize().unwrap();
    assert_eq!(final_stmt.operators, vec![BinaryOp::Add]);
}

#[test]
fn test_assembler_numeral_value() {
    let mut asm = Assembler::new();
    let analysis = analyze("πέντε"); // "five"
    asm.feed(&analysis, "πέντε").unwrap();
    let final_stmt = asm.finalize().unwrap();
    assert_eq!(final_stmt.literals.len(), 1);
    if let Literal::Number(n) = final_stmt.literals[0] {
        assert_eq!(n, 5);
    } else {
        panic!("Expected Number literal");
    }
}

#[test]
fn test_assembler_coverage_missing_verb_complex_expr() -> Result<(), glossa::errors::GlossaError> {
    let mut asm = glossa::semantic::assembly::Assembler::new();
    let subj = glossa::morphology::analyze_all("ανθρωπος");
    let _ = asm.feed(&subj[0], "ἄνθρωπος");
    let obj = glossa::morphology::analyze_all("λογον");
    let _ = asm.feed(&obj[0], "λόγον");
    let res = asm.finalize();
    assert!(res.is_err());
    Ok(())
}

#[test]
fn test_assembler_coverage_missing_verb_complex_expr_obj() -> Result<(), glossa::errors::GlossaError>
{
    let mut asm = glossa::semantic::assembly::Assembler::new();
    let obj = glossa::morphology::analyze_all("λογον");
    let _ = asm.feed(&obj[0], "λόγον");
    let subj = glossa::morphology::analyze_all("ανθρωπος");
    let _ = asm.feed(&subj[0], "ἄνθρωπος");
    let res = asm.finalize();
    assert!(res.is_err());
    Ok(())
}

#[test]
fn test_conversion_undefined_unknown_type() {
    let source = "ἄγνωστος λέγε.";
    let ast = glossa::parser::parse(source).unwrap();
    let prog = glossa::semantic::analyze_program(&ast);
    assert!(prog.is_err());
}

#[test]
fn test_conversion_undefined_unknown_type_try_print() {
    let source = "ἄγνωστος λέγε.";
    let ast = glossa::parser::parse(source).unwrap();
    let prog = glossa::semantic::analyze_program(&ast);
    assert!(prog.is_err());
}
