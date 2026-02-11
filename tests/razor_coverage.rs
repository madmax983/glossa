use glossa::codegen::to_rust_type;
use glossa::semantic::{
    AssembledStatement, Constituent, GlossaType, Literal, ParticipleConstituent, VerbConstituent,
};
#[test]
fn test_glossa_type_display_coverage() {
    // Exercise Display impl for all variants to ensure full coverage
    let types = vec![
        GlossaType::Number,
        GlossaType::String,
        GlossaType::Boolean,
        GlossaType::Unit,
        GlossaType::Unknown,
        GlossaType::List(Box::new(GlossaType::Number)),
        GlossaType::Set(Box::new(GlossaType::String)),
        GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number)),
        GlossaType::Option(Box::new(GlossaType::Number)),
        GlossaType::Result(Box::new(GlossaType::Unit), Box::new(GlossaType::String)),
        GlossaType::Struct {
            name: "User".into(),
            gender: glossa::morphology::Gender::Masculine,
            fields: vec![],
        },
        GlossaType::Function {
            params: vec![GlossaType::Number],
            returns: Box::new(GlossaType::Boolean),
        },
    ];

    for ty in types {
        let _ = format!("{}", ty);
        let _ = ty.to_greek();
        // Also exercise to_rust_type
        let _ = to_rust_type(&ty);
    }
}

#[test]
fn test_assembled_statement_derives() {
    // Exercise Debug, Clone, Default for AssembledStatement and children
    let stmt = AssembledStatement::default();
    let _ = format!("{:?}", stmt);
    let clone = stmt.clone();
    let _ = format!("{:?}", clone);

    let lit = Literal::String("test".to_string());
    let _ = format!("{:?}", lit);
    let _ = lit.clone();

    let constituent = Constituent {
        lemma: "word".into(),
        original: "word".into(),
        case: glossa::morphology::Case::Nominative,
        number: None,
        gender: None,
        person: None,
    };
    let _ = format!("{:?}", constituent);
    let _ = constituent.clone();

    let verb = VerbConstituent {
        lemma: "verb".into(),
        original: "verb".into(),
        person: None,
        number: None,
        tense: None,
        mood: None,
        voice: None,
    };
    let _ = format!("{:?}", verb);
    let _ = verb.clone();

    let part = ParticipleConstituent {
        verb_lemma: "verb".into(),
        original: "verb".into(),
        tense: glossa::morphology::Tense::Present,
        voice: glossa::morphology::Voice::Active,
        case: glossa::morphology::Case::Nominative,
        gender: glossa::morphology::Gender::Masculine,
        number: glossa::morphology::Number::Singular,
    };
    let _ = format!("{:?}", part);
    let _ = part.clone();
}
