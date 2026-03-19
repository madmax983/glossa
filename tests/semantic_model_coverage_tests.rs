use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedMethod, AnalyzedStatement, AssembledStatement,
    CaptureMode, Constituent, Literal, ParticipleConstituent, TraitDef, TraitImpl, VerbConstituent,
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

    let stmt3 = AnalyzedStatement::Return { value: None };
    let dbg3 = format!("{:?}", stmt3);
    assert!(dbg3.contains("Return"));
    assert!(dbg3.contains("value"));

    let stmt4 = AnalyzedStatement::If {
        condition: Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: glossa::semantic::GlossaType::Boolean,
        }),
        then_body: vec![],
        else_body: None,
    };
    let dbg4 = format!("{:?}", stmt4);
    assert!(dbg4.contains("If"));
    assert!(dbg4.contains("condition"));

    let stmt5 = AnalyzedStatement::TypeDefinition {
        name: SmolStr::new("Type"),
        fields: vec![],
    };
    let dbg5 = format!("{:?}", stmt5);
    assert!(dbg5.contains("TypeDefinition"));

    let stmt6 = AnalyzedStatement::TraitDefinition {
        name: SmolStr::new("Trait"),
        methods: vec![],
    };
    let dbg6 = format!("{:?}", stmt6);
    assert!(dbg6.contains("TraitDefinition"));

    let stmt_binding = AnalyzedStatement::Binding {
        name: SmolStr::new("name"),
        value: AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: glossa::semantic::GlossaType::Unknown,
        },
        mutable: false,
    };
    assert!(format!("{:?}", stmt_binding).contains("Binding"));

    let stmt_assignment = AnalyzedStatement::Assignment {
        name: SmolStr::new("name"),
        value: AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: glossa::semantic::GlossaType::Unknown,
        },
    };
    assert!(format!("{:?}", stmt_assignment).contains("Assignment"));

    let stmt_print = AnalyzedStatement::Print(vec![]);
    assert!(format!("{:?}", stmt_print).contains("Print"));

    let stmt_expr = AnalyzedStatement::Expression(vec![]);
    assert!(format!("{:?}", stmt_expr).contains("Expression"));

    let stmt_query = AnalyzedStatement::Query(vec![]);
    assert!(format!("{:?}", stmt_query).contains("Query"));

    let stmt_while = AnalyzedStatement::While {
        condition: Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: glossa::semantic::GlossaType::Unknown,
        }),
        body: vec![],
    };
    assert!(format!("{:?}", stmt_while).contains("While"));

    let stmt_for = AnalyzedStatement::For {
        variable: SmolStr::new("var"),
        iterator: Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: glossa::semantic::GlossaType::Unknown,
        }),
        body: vec![],
    };
    assert!(format!("{:?}", stmt_for).contains("For"));

    let stmt_match = AnalyzedStatement::Match {
        scrutinee: Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: glossa::semantic::GlossaType::Unknown,
        }),
        arms: vec![],
    };
    assert!(format!("{:?}", stmt_match).contains("Match"));

    let stmt_func = AnalyzedStatement::FunctionDef {
        name: SmolStr::new("func"),
        params: vec![],
        body: vec![],
        return_type: None,
    };
    assert!(format!("{:?}", stmt_func).contains("FunctionDef"));

    let stmt_trait_impl = AnalyzedStatement::TraitImplementation {
        trait_name: SmolStr::new("trait"),
        type_name: SmolStr::new("type"),
        methods: vec![],
    };
    assert!(format!("{:?}", stmt_trait_impl).contains("TraitImplementation"));

    let stmt_test = AnalyzedStatement::TestDeclaration {
        name: "test".to_string(),
        body: vec![],
    };
    assert!(format!("{:?}", stmt_test).contains("TestDeclaration"));
}

#[test]
fn test_analyzed_expr_kind_debug_all_variants() {
    let variants = vec![
        AnalyzedExprKind::StringLiteral("test".to_string()),
        AnalyzedExprKind::NumberLiteral(1),
        AnalyzedExprKind::BooleanLiteral(true),
        AnalyzedExprKind::Variable(SmolStr::new("x")),
        AnalyzedExprKind::PropertyAccess {
            owner: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
            property: SmolStr::new("prop"),
        },
        AnalyzedExprKind::VerbCall {
            verb: SmolStr::new("verb"),
            args: vec![],
        },
        AnalyzedExprKind::BinOp {
            left: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
            op: glossa::morphology::lexicon::BinaryOp::Add,
            right: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
        },
        AnalyzedExprKind::UnaryOp {
            op: glossa::morphology::lexicon::UnaryOp::Not,
            operand: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
        },
        AnalyzedExprKind::Range {
            start: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
            end: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
            inclusive: false,
        },
        AnalyzedExprKind::ArrayLiteral(vec![]),
        AnalyzedExprKind::Some(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: glossa::semantic::GlossaType::Unknown,
        })),
        AnalyzedExprKind::None,
        AnalyzedExprKind::Ok(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: glossa::semantic::GlossaType::Unknown,
        })),
        AnalyzedExprKind::Err(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: glossa::semantic::GlossaType::Unknown,
        })),
        AnalyzedExprKind::Unwrap(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: glossa::semantic::GlossaType::Unknown,
        })),
        AnalyzedExprKind::Try(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: glossa::semantic::GlossaType::Unknown,
        })),
        AnalyzedExprKind::IndexAccess {
            array: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
            index: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
        },
        AnalyzedExprKind::FunctionCall {
            func: SmolStr::new("func"),
            args: vec![],
        },
        AnalyzedExprKind::MethodCall {
            receiver: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
            method: SmolStr::new("method"),
            args: vec![],
        },
        AnalyzedExprKind::TraitMethodCall {
            receiver: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
            trait_name: SmolStr::new("Trait"),
            method_name: SmolStr::new("method"),
            args: vec![],
        },
        AnalyzedExprKind::StructInstantiation {
            type_name: SmolStr::new("Type"),
            fields: vec![],
            args: vec![],
        },
        AnalyzedExprKind::Lambda {
            params: vec![],
            body: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
            capture_mode: CaptureMode::Borrow,
        },
        AnalyzedExprKind::CollectionNew {
            collection_type: "list".to_string(),
        },
        AnalyzedExprKind::Assert {
            condition: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
        },
        AnalyzedExprKind::AssertEq {
            left: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
            right: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: glossa::semantic::GlossaType::Unknown,
            }),
        },
    ];

    for variant in variants {
        let dbg = format!("{:?}", variant);
        assert!(!dbg.is_empty());
    }
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
