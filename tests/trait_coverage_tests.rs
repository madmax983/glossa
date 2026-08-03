use glossa::ast::{Statement, MethodDef, Word};
use glossa::semantic::{analyze_statement, Scope, GlossaType, TraitDef, AnalyzedMethod};
use glossa::morphology::Gender;

#[test]
fn test_impl_method_none_body() {
    let mut scope = Scope::new();
    let trait_impl = Statement::TraitImpl(glossa::ast::TraitImplDef {
        trait_name: Word { original: "Tr".into(), normalized: "Tr".into() },
        type_name: Word { original: "T".into(), normalized: "T".into() },
        methods: vec![
            MethodDef {
                name: Word { original: "meth".into(), normalized: "meth".into() },
                params: vec![],
                is_default: false,
                body: None, // This tests the None branch
            }
        ]
    });

    // define struct T and trait Tr
    scope.define_type("T", GlossaType::Struct { name: "T".into(), fields: vec![], gender: Gender::Neuter });
    scope.define_trait("Tr", TraitDef { name: "Tr".into(), methods: vec![AnalyzedMethod { name: "meth".into(), params: vec![], body: None, return_type: None }] });

    let _ = analyze_statement(&trait_impl, &mut scope);
}
