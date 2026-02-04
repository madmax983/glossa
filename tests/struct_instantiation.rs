use glossa::parser::parse;
use glossa::semantic::{analyze_program, StatementKind, AnalyzedExprKind};

#[test]
fn test_struct_instantiation_with_variable_args() {
    // Define struct Point { x: i64, y: i64 }
    // let a = 10;
    // let p = new Point(a, 20);

    // In Glossa:
    // εἶδος Σημεῖον ὁρίζειν { χ ἀριθμοῦ. ψ ἀριθμοῦ. }
    // α 10 ἔστω.
    // π νέον Σημεῖον α 20 ἔστω.

    let code = "
    εἶδος Σημεῖον ὁρίζειν { χ ἀριθμοῦ. ψ ἀριθμοῦ. }.
    α 10 ἔστω.
    π νέον Σημεῖον α 20 ἔστω.
    ";

    let ast = parse(code).unwrap();
    let analyzed = analyze_program(&ast).unwrap();

    // 3 statements: TypeDef, Binding(a), Binding(p)
    println!("Analyzed statements count: {}", analyzed.statements.len());
    for (i, s) in analyzed.statements.iter().enumerate() {
        println!("Stmt {}: {:?}", i, s.kind);
    }
    assert_eq!(analyzed.statements.len(), 3);

    let stmt = &analyzed.statements[2];
    if let StatementKind::Binding { name, .. } = &stmt.kind {
        assert_eq!(name, "π");
    } else {
        panic!("Expected Binding statement, got {:?}", stmt.kind);
    }

    // Check args
    let exprs = &stmt.expressions;
    // expressions[0] is the variable 'p', expressions[1] is the instantiation
    if let AnalyzedExprKind::StructInstantiation { args, .. } = &exprs[1].expr {
        assert_eq!(args.len(), 2);
        // First arg should be variable 'α'
        match &args[0].expr {
            AnalyzedExprKind::Variable(v) => assert_eq!(v, "α"),
            _ => panic!("Expected variable 'α', got {:?}", args[0].expr),
        }
        // Second arg should be literal 20
        match &args[1].expr {
            AnalyzedExprKind::NumberLiteral(n) => assert_eq!(*n, 20),
            _ => panic!("Expected literal 20, got {:?}", args[1].expr),
        }
    } else {
        panic!("Expected StructInstantiation, got {:?}", exprs[1].expr);
    }
}
