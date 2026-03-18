use glossa::ast::Word;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedMethod, AnalyzedStatement, CaptureMode, TraitDef,
    TraitImpl, AssembledStatement, Constituent, VerbConstituent, ParticipleConstituent, Literal
};
use smol_str::SmolStr;

#[test]
fn test_analyzed_statement_debug() {
    let stmt = AnalyzedStatement::Break;
    let dbg = format!("{:?}", stmt);
    assert!(dbg.contains("Break"));

    let stmt2 = AnalyzedStatement::Continue;
    let dbg2 = format!("{:?}", stmt2);
    assert!(dbg2.contains("Continue"));
}

#[test]
fn test_analyzed_expr_debug() {
    let expr = AnalyzedExpr {
        expr: AnalyzedExprKind::None,
        glossa_type: glossa::semantic::GlossaType::Unknown,
    };
    let dbg = format!("{:?}", expr);
    assert!(dbg.contains("AnalyzedExpr"));
    assert!(dbg.contains("None"));
}

#[test]
fn test_analyzed_expr_kind_debug() {
    let expr = AnalyzedExprKind::BooleanLiteral(true);
    let dbg = format!("{:?}", expr);
    assert!(dbg.contains("BooleanLiteral"));
    assert!(dbg.contains("true"));
}

#[test]
fn test_analyzed_method_debug() {
    let method = AnalyzedMethod {
        name: SmolStr::new("test"),
        params: vec![],
        body: None,
        return_type: None,
    };
    let dbg = format!("{:?}", method);
    assert!(dbg.contains("AnalyzedMethod"));
    assert!(dbg.contains("test"));
}

#[test]
fn test_trait_def_debug() {
    let def = TraitDef {
        name: SmolStr::new("Trait"),
        methods: vec![],
    };
    let dbg = format!("{:?}", def);
    assert!(dbg.contains("TraitDef"));
}

#[test]
fn test_trait_impl_debug() {
    let impl_def = TraitImpl {
        trait_name: SmolStr::new("Trait"),
        type_name: SmolStr::new("Type"),
    };
    let dbg = format!("{:?}", impl_def);
    assert!(dbg.contains("TraitImpl"));
}

#[test]
fn test_assembled_statement_debug() {
    let stmt = AssembledStatement::default();
    let dbg = format!("{:?}", stmt);
    assert!(dbg.contains("AssembledStatement"));
}

#[test]
fn test_constituent_debug() {
    let constituent = Constituent {
        lemma: SmolStr::new("test"),
        original: SmolStr::new("test"),
        normalized: SmolStr::new("test"),
        case: glossa::morphology::Case::Nominative,
        number: None,
        gender: None,
        person: None,
    };
    let dbg = format!("{:?}", constituent);
    assert!(dbg.contains("Constituent"));
}

#[test]
fn test_verb_constituent_debug() {
    let verb = VerbConstituent {
        lemma: SmolStr::new("test"),
        original: SmolStr::new("test"),
        normalized: SmolStr::new("test"),
        person: None,
        number: None,
        tense: None,
        mood: None,
        voice: None,
    };
    let dbg = format!("{:?}", verb);
    assert!(dbg.contains("VerbConstituent"));
}

#[test]
fn test_participle_constituent_debug() {
    let part = ParticipleConstituent {
        verb_lemma: SmolStr::new("test"),
        original: SmolStr::new("test"),
        normalized: SmolStr::new("test"),
        tense: glossa::morphology::Tense::Present,
        voice: glossa::morphology::Voice::Active,
        case: glossa::morphology::Case::Nominative,
        gender: glossa::morphology::Gender::Masculine,
        number: glossa::morphology::Number::Singular,
    };
    let dbg = format!("{:?}", part);
    assert!(dbg.contains("ParticipleConstituent"));
}

#[test]
fn test_literal_debug() {
    let lit = Literal::Boolean(true);
    let dbg = format!("{:?}", lit);
    assert!(dbg.contains("Boolean"));
}

#[test]
fn test_deep_recursion_debug() {
    // Create a deeply nested AnalyzedExpr
    let mut expr = AnalyzedExprKind::None;
    for _ in 0..100 {
        expr = AnalyzedExprKind::Unwrap(Box::new(AnalyzedExpr {
            expr,
            glossa_type: glossa::semantic::GlossaType::Unknown,
        }));
    }
    let dbg = format!("{:?}", expr);
    assert!(dbg.contains("Unwrap"));
}
