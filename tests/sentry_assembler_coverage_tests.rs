use glossa::morphology::analyze;
use glossa::morphology::lexicon::BinaryOp;
use glossa::semantic::Assembler;
use glossa::semantic::assembly::Literal;

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
