#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::{AnalyzedExprKind, AnalyzedStatement, analyze_program};

#[test]
fn test_struct_instantiation_with_variable_args() {
    let code = "
    εἶδος Σημεῖον ὁρίζειν { χ ἀριθμοῦ. ψ ἀριθμοῦ. }.
    α 10 ἔστω.
    π νέον Σημεῖον α 20 ἔστω.
    ";

    let ast = parse(code).unwrap();
    let analyzed = analyze_program(&ast).unwrap();

    assert_eq!(analyzed.statements.len(), 3);

    let stmt = &analyzed.statements[2];
    if let AnalyzedStatement::Binding { name, value, .. } = stmt {
        assert_eq!(name, "π");

        if let AnalyzedExprKind::StructInstantiation { args, .. } = &value.expr {
            assert_eq!(args.len(), 2);
            match &args[0].expr {
                AnalyzedExprKind::Variable(v) => assert_eq!(v, "α"),
                _ => panic!("Expected variable 'α', got {:?}", args[0].expr),
            }
            match &args[1].expr {
                AnalyzedExprKind::NumberLiteral(n) => assert_eq!(*n, 20),
                _ => panic!("Expected literal 20, got {:?}", args[1].expr),
            }
        } else {
            panic!("Expected StructInstantiation, got {:?}", value.expr);
        }
    } else {
        panic!("Expected Binding statement, got {:?}", stmt);
    }
}

#[test]
fn test_try_parse_struct_instantiation_unsupported_args() {
    use glossa::parser::parse;
    use glossa::semantic::analyze_program;

    // Trigger unsupported argument logic to parse_single_struct_arg
    // We'll pass a boolean for an argument expecting a number to verify boolean processing.
    // Also, we use a string literal to hit the string argument processing branch when a number is expected.
    let code = "εἶδος Α ὁρίζειν { β ἀριθμοῦ. }. γ νέον Α ἀληθές ἔστω. δ νέον Α «κείμενον» ἔστω.";
    let ast = parse(code).expect("Should parse");
    let analyzed = analyze_program(&ast).expect("Should analyze");
    assert!(!analyzed.statements.is_empty());
}
