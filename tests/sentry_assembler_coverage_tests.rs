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
fn test_double_indirect_object_error() {
    let mut asm = Assembler::new();
    let obj1 = analyze("ἀνθρώπῳ");
    asm.feed(&obj1, "ἀνθρώπῳ").unwrap();
    let obj2 = analyze("θεῷ");
    let result = asm.feed(&obj2, "θεῷ");
    assert!(matches!(
        result,
        Err(glossa::errors::AssemblyError::DoubleIndirect)
    ));
}

#[test]
fn test_missing_verb_error() {
    let mut asm = Assembler::new();
    let subj = analyze("ἄνθρωπος");
    asm.feed(&subj, "ἄνθρωπος").unwrap();
    let obj = analyze("λόγον");
    asm.feed(&obj, "λόγον").unwrap();
    let result = asm.finalize();
    assert!(matches!(
        result,
        Err(glossa::errors::AssemblyError::MissingVerb)
    ));
}

#[test]
fn test_double_subject_no_verb() {
    let mut asm = Assembler::new();
    let subj1 = analyze("ἄνθρωπος");
    asm.feed(&subj1, "ἄνθρωπος").unwrap();
    let subj2 = analyze("θεός");
    asm.feed(&subj2, "θεός").unwrap();
    let fin_result = asm.finalize();
    assert!(matches!(
        fin_result,
        Err(glossa::errors::AssemblyError::DoubleSubject)
    ));
}
