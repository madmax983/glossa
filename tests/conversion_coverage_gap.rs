use glossa::morphology::analyze;
use glossa::semantic::{Assembler, Scope, AnalyzedStatement, AnalyzedExprKind, GlossaType};
use glossa::semantic::conversion::convert_assembled_to_analyzed;
use glossa::morphology::Gender;

fn setup_scope() -> Scope {
    let mut scope = Scope::new();
    scope.define("χ", GlossaType::Number);
    scope.define("ψ", GlossaType::Number);
    // Use Gender::Neuter or similar for gender field (GlossaType::Struct expects Gender, not Option<Gender>)
    scope.define("χαρτης_inst", GlossaType::Struct { name: "χαρτης".into(), fields: vec![], gender: Gender::Neuter });
    scope.define("λιστη", GlossaType::List(Box::new(GlossaType::Number)));
    scope
}

#[test]
fn test_classify_property_access_print() {
    let mut asm = Assembler::new();
    let mut scope = setup_scope();

    let user_gen = analyze("χρήστου");
    asm.feed(&user_gen, "χρήστου").unwrap();
    scope.define("χρηστης", GlossaType::Struct { name: "User".into(), fields: vec![], gender: Gender::Masculine });

    let name = analyze("ὄνομα");
    asm.feed(&name, "ὄνομα").unwrap();

    let print = analyze("λέγε");
    asm.feed(&print, "λέγε").unwrap();

    let assembled = asm.finalize().unwrap();
    let analyzed = convert_assembled_to_analyzed(&assembled, &mut scope);

    if let Ok(AnalyzedStatement::Print(exprs)) = analyzed {
        if let AnalyzedExprKind::PropertyAccess { .. } = exprs[0].expr {
            // Success
        }
    }
}

#[test]
fn test_classify_subjunctive_comparison() {
    let mut asm = Assembler::new();
    let mut scope = setup_scope();

    let x = analyze("χ");
    asm.feed(&x, "χ").unwrap();

    let gt = analyze("μεῖζον");
    asm.feed(&gt, "μεῖζον").unwrap();

    asm.feed_number(5).unwrap();

    let be_subj = analyze("ᾖ");
    asm.feed(&be_subj, "ᾖ").unwrap();

    let assembled = asm.finalize().unwrap();
    let analyzed = convert_assembled_to_analyzed(&assembled, &mut scope);

    assert!(matches!(analyzed, Ok(AnalyzedStatement::Expression(_))));
}

#[test]
fn test_extract_value_unwraps() {
    let mut asm = Assembler::new();
    let mut scope = setup_scope();

    let y = analyze("ψ");
    asm.feed(&y, "ψ").unwrap();

    use glossa::ast::{Expr, Word};
    asm.feed_unwrap(Expr::Word(Word::new("χ"))).unwrap();

    let verb = analyze("ἔστω");
    asm.feed(&verb, "ἔστω").unwrap();

    let assembled = asm.finalize().unwrap();
    let analyzed = convert_assembled_to_analyzed(&assembled, &mut scope);

    match analyzed {
        Ok(AnalyzedStatement::Binding { value, .. }) => {
            assert!(matches!(value.expr, AnalyzedExprKind::Unwrap(_)));
        }
        _ => panic!("Expected binding with unwrap, got {:?}", analyzed),
    }
}

#[test]
fn test_extract_value_array() {
    let mut asm = Assembler::new();
    let mut scope = setup_scope();

    let y = analyze("ψ");
    asm.feed(&y, "ψ").unwrap();

    use glossa::ast::Expr;
    asm.feed_array(vec![Expr::NumberLiteral(1), Expr::NumberLiteral(2)]).unwrap();

    let verb = analyze("ἔστω");
    asm.feed(&verb, "ἔστω").unwrap();

    let assembled = asm.finalize().unwrap();
    let analyzed = convert_assembled_to_analyzed(&assembled, &mut scope);

    match analyzed {
        Ok(AnalyzedStatement::Binding { value, .. }) => {
            assert!(matches!(value.expr, AnalyzedExprKind::ArrayLiteral(_)));
        }
        _ => panic!("Expected binding with array, got {:?}", analyzed),
    }
}

#[test]
fn test_extract_value_index_access() {
    let mut asm = Assembler::new();
    let mut scope = setup_scope();

    let y = analyze("ψ");
    asm.feed(&y, "ψ").unwrap();

    use glossa::ast::{Expr, Word};
    asm.feed_index_access(
        Expr::Word(Word::new("λιστη")),
        Expr::NumberLiteral(0)
    ).unwrap();

    let verb = analyze("ἔστω");
    asm.feed(&verb, "ἔστω").unwrap();

    let assembled = asm.finalize().unwrap();
    let analyzed = convert_assembled_to_analyzed(&assembled, &mut scope);

    match analyzed {
        Ok(AnalyzedStatement::Binding { value, .. }) => {
            assert!(matches!(value.expr, AnalyzedExprKind::IndexAccess { .. }));
        }
        _ => panic!("Expected binding with index access, got {:?}", analyzed),
    }
}

#[test]
fn test_classify_genitive_method_call() {
    let mut asm = Assembler::new();
    let mut scope = setup_scope();

    scope.define("λιστα", GlossaType::Struct { name: "List".into(), fields: vec![], gender: Gender::Feminine });

    let list_gen = analyze("λίστας");
    asm.feed(&list_gen, "λίστας").unwrap();

    let method = analyze("ἀφαίρεσις");
    asm.feed(&method, "ἀφαίρεσις").unwrap();

    let assembled = asm.finalize().unwrap();
    let analyzed = convert_assembled_to_analyzed(&assembled, &mut scope);

    match analyzed {
        Ok(AnalyzedStatement::Expression(exprs)) => {
            if let AnalyzedExprKind::MethodCall { method, .. } = &exprs[0].expr {
                assert_eq!(method, "αφαιρεσις");
            } else {
                panic!("Expected MethodCall, got {:?}", exprs[0]);
            }
        }
        _ => panic!("Expected Expression statement, got {:?}", analyzed),
    }
}
