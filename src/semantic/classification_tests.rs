use super::conversion::classify_assembled_statement;
use crate::morphology::Case;
use crate::semantic::assembly_model::{ParticipleConstituent, VerbConstituent};
use crate::semantic::{
    AnalyzedExprKind, AnalyzedStatement, AssembledStatement, Constituent, GlossaType, Literal, Scope,
};
use crate::text::normalize_greek;
use crate::morphology::lexicon::BinaryOp;

fn make_constituent(original: &str, lemma: &str) -> Constituent {
    Constituent {
        lemma: lemma.into(),
        original: original.into(),
        normalized: normalize_greek(original),
        case: Case::Nominative,
        number: None,
        gender: None,
        person: None,
    }
}

fn make_verb(original: &str, lemma: &str) -> VerbConstituent {
    VerbConstituent {
        lemma: lemma.into(),
        original: original.into(),
        normalized: normalize_greek(original),
        person: None,
        number: None,
        tense: None,
        mood: None,
        voice: None,
    }
}

#[test]
fn test_classify_simple_binding() {
    let mut scope = Scope::new();

    // "x 5 let"
    let mut asm_stmt = AssembledStatement::default();
    asm_stmt.subject = Some(make_constituent("x", "x"));
    asm_stmt.literals = vec![Literal::Number(5)];
    asm_stmt.verb = Some(make_verb("let", "εστω")); // "ἔστω" is a binding verb

    let result = classify_assembled_statement(&asm_stmt, &mut scope)
        .expect("Classification failed");

    if let AnalyzedStatement::Binding { name, value, .. } = result {
        assert_eq!(name, "x");
        if let AnalyzedExprKind::NumberLiteral(n) = value.expr {
            assert_eq!(n, 5);
        } else {
            panic!("Expected NumberLiteral");
        }
    } else {
        panic!("Expected Binding, got {:?}", result);
    }
}

#[test]
fn test_classify_binding_subject_object_swap() {
    let mut scope = Scope::new();
    scope.define("val", GlossaType::Number);

    // "val x let" -> Should bind x to val, because val is defined and x is not.
    // Original: Subject=val, Object=x (because of word order/case, usually)
    // Here we simulate the assembler putting "val" in Subject and "x" in Object.
    let mut asm_stmt = AssembledStatement::default();
    asm_stmt.subject = Some(make_constituent("val", "val"));
    asm_stmt.object = Some(make_constituent("x", "x"));
    asm_stmt.verb = Some(make_verb("let", "εστω"));

    let result = classify_assembled_statement(&asm_stmt, &mut scope)
        .expect("Classification failed");

    if let AnalyzedStatement::Binding { name, value, .. } = result {
        assert_eq!(name, "x"); // Should be swapped to x
        if let AnalyzedExprKind::Variable(v) = value.expr {
            assert_eq!(v, "val");
        } else {
            panic!("Expected Variable val");
        }
    } else {
        panic!("Expected Binding, got {:?}", result);
    }
}

#[test]
fn test_classify_binding_false_participle() {
    let mut scope = Scope::new();

    // "x 5 written let" -> "written" is a participle but not a real one in this context (it's the variable name essentially or part of the phrase)
    // The logic checks if the participle lemma exists in lexicon. If not, it treats it as the variable name.

    let mut asm_stmt = AssembledStatement::default();
    asm_stmt.literals = vec![Literal::Number(5)];
    asm_stmt.verb = Some(make_verb("let", "εστω"));

    // Add a "false" participle (lemma not in lexicon)
    asm_stmt.participles.push(ParticipleConstituent {
        verb_lemma: "unknown_verb".into(),
        original: "written".into(),
        normalized: "written".into(),
        tense: crate::morphology::Tense::Present, // Dummy values
        voice: crate::morphology::Voice::Active,
        case: Case::Nominative,
        gender: crate::morphology::Gender::Neuter,
        number: crate::morphology::Number::Singular,
    });

    let result = classify_assembled_statement(&asm_stmt, &mut scope)
        .expect("Classification failed");

    if let AnalyzedStatement::Binding { name, .. } = result {
        assert_eq!(name, "written"); // Should bind to the participle's normalized form
    } else {
        panic!("Expected Binding, got {:?}", result);
    }
}

#[test]
fn test_classify_print_binary_op() {
    let mut scope = Scope::new();
    scope.define("x", GlossaType::Number);

    // "x + 5 print"
    let mut asm_stmt = AssembledStatement::default();
    asm_stmt.subject = Some(make_constituent("x", "x"));
    asm_stmt.literals = vec![Literal::Number(5)];
    asm_stmt.operators = vec![BinaryOp::Add];
    asm_stmt.verb = Some(make_verb("print", "λεγε"));

    let result = classify_assembled_statement(&asm_stmt, &mut scope)
        .expect("Classification failed");

    if let AnalyzedStatement::Print(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::BinOp { left, op, right } = &exprs[0].expr {
            assert_eq!(*op, BinaryOp::Add);
            // check left is variable x
             if let AnalyzedExprKind::Variable(v) = &left.expr {
                assert_eq!(v, "x");
            } else {
                panic!("Expected Variable x");
            }
        } else {
            panic!("Expected BinOp");
        }
    } else {
        panic!("Expected Print, got {:?}", result);
    }
}

#[test]
fn test_classify_print_property_access() {
    let mut scope = Scope::new();
    scope.define("user", GlossaType::Unknown); // Type doesn't matter much for lookup unless it fails

    // "user.name print"
    // Assembler represents this as property_accesses: [("user", "name")]
    let mut asm_stmt = AssembledStatement::default();
    asm_stmt.property_accesses.push(("user".into(), "name".into()));
    asm_stmt.verb = Some(make_verb("print", "λεγε"));

    let result = classify_assembled_statement(&asm_stmt, &mut scope)
        .expect("Classification failed");

    if let AnalyzedStatement::Print(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::MethodCall { receiver, method, .. } = &exprs[0].expr {
            assert_eq!(method, "name");
            if let AnalyzedExprKind::Variable(v) = &receiver.expr {
                assert_eq!(v, "user");
            } else {
                panic!("Expected Variable user");
            }
        } else {
            panic!("Expected MethodCall (property access is often lowered to method call or specific enum)");
        }
    } else {
        panic!("Expected Print, got {:?}", result);
    }
}
