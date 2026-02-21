use super::conversion::classify_assembled_statement;
use crate::morphology::{Case, Gender, Number, Tense, Voice};
use crate::semantic::assembly_model::{
    AssembledStatement, Constituent, ParticipleConstituent, VerbConstituent,
};
use crate::semantic::{AnalyzedExprKind, AnalyzedStatement, GlossaType, Literal, Scope};

fn make_constituent(original: &str, lemma: &str) -> Constituent {
    Constituent {
        lemma: lemma.into(),
        original: original.into(),
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
        person: None,
        number: None,
        tense: None,
        mood: None,
        voice: None,
    }
}

fn make_participle(original: &str, verb_lemma: &str) -> ParticipleConstituent {
    ParticipleConstituent {
        verb_lemma: verb_lemma.into(),
        original: original.into(),
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Neuter,
        number: Number::Singular,
    }
}

#[test]
fn test_classify_variable_binding_simple() {
    let mut scope = Scope::new();
    // "x 5 let"
    let asm_stmt = AssembledStatement {
        subject: Some(make_constituent("x", "x")),
        literals: vec![Literal::Number(5)],
        verb: Some(make_verb("ἔστω", "ειμι")),
        ..Default::default()
    };

    let result =
        classify_assembled_statement(&asm_stmt, &mut scope).expect("Should classify binding");

    if let AnalyzedStatement::Binding { name, value, .. } = result {
        assert_eq!(name, "x");
        if let AnalyzedExprKind::NumberLiteral(n) = value.expr {
            assert_eq!(n, 5);
        } else {
            panic!("Expected NumberLiteral");
        }
    } else {
        panic!("Expected Binding statement");
    }
}

#[test]
fn test_classify_variable_binding_swap_subject_object() {
    let mut scope = Scope::new();
    scope.define("y", GlossaType::Number); // y is defined

    // "x y let" (x is new, y is existing)
    // If assembler puts x in Subject and y in Object (or vice versa), logic should handle it.
    // Logic: if subject is defined and object is not, swap.

    // Case 1: Subject=y (defined), Object=x (undefined) -> Should bind x to y
    let asm_stmt = AssembledStatement {
        subject: Some(make_constituent("y", "y")),
        object: Some(make_constituent("x", "x")),
        verb: Some(make_verb("ἔστω", "ειμι")),
        ..Default::default()
    };

    let result = classify_assembled_statement(&asm_stmt, &mut scope)
        .expect("Should classify binding with swap");

    if let AnalyzedStatement::Binding { name, value, .. } = result {
        assert_eq!(name, "x"); // Bound variable
        if let AnalyzedExprKind::Variable(v) = value.expr {
            assert_eq!(v, "y"); // Value source
        } else {
            panic!("Expected Variable value");
        }
    } else {
        panic!("Expected Binding statement");
    }
}

#[test]
fn test_classify_variable_binding_false_participle() {
    let mut scope = Scope::new();

    // "variable called x let"
    // "called" (participle) might be treated as part of the name if it's not a known verb?
    // The logic checks for "false participle" (not in lexicon) and treats it as the name.

    let asm_stmt = AssembledStatement {
        participles: vec![make_participle("x", "x_lemma")], // "x" as participle?
        literals: vec![Literal::Number(10)],
        verb: Some(make_verb("ἔστω", "ειμι")),
        ..Default::default()
    };

    // x_lemma is not in lexicon (hopefully), so it should be treated as the variable name

    let result = classify_assembled_statement(&asm_stmt, &mut scope)
        .expect("Should classify binding with false participle");

    if let AnalyzedStatement::Binding { name, .. } = result {
        assert_eq!(name, "x");
    } else {
        panic!("Expected Binding statement");
    }
}

#[test]
fn test_classify_print_binary() {
    let mut scope = Scope::new();

    // "1 + 2 print"
    let asm_stmt = AssembledStatement {
        literals: vec![Literal::Number(1), Literal::Number(2)],
        operators: vec![crate::morphology::lexicon::BinaryOp::Add],
        verb: Some(make_verb("λέγε", "λεγω")),
        ..Default::default()
    };

    let result =
        classify_assembled_statement(&asm_stmt, &mut scope).expect("Should classify print binary");

    if let AnalyzedStatement::Print(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::BinOp { op, .. } = exprs[0].expr {
            assert_eq!(op, crate::morphology::lexicon::BinaryOp::Add);
        } else {
            panic!("Expected BinOp");
        }
    } else {
        panic!("Expected Print statement");
    }
}

#[test]
fn test_classify_print_property() {
    let mut scope = Scope::new();
    scope.define("list", GlossaType::List(Box::new(GlossaType::Number)));

    // "list length print" (property access)
    // property accesses are pre-calculated by assembler usually, but classify_print checks asm_stmt.property_accesses
    let asm_stmt = AssembledStatement {
        property_accesses: vec![("list".into(), "len".into())],
        verb: Some(make_verb("λέγε", "λεγω")),
        ..Default::default()
    };

    let result = classify_assembled_statement(&asm_stmt, &mut scope)
        .expect("Should classify print property");

    if let AnalyzedStatement::Print(exprs) = result {
        assert_eq!(exprs.len(), 1);
        if let AnalyzedExprKind::MethodCall { method, .. } = &exprs[0].expr {
            assert_eq!(method, "len");
        } else {
            panic!("Expected MethodCall for property access");
        }
    } else {
        panic!("Expected Print statement");
    }
}
