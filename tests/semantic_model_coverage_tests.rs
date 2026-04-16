use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedMethod, AnalyzedStatement, CaptureMode, GlossaType,
};
use smol_str::SmolStr;
use std::collections::HashMap;

#[test]
fn test_analyzed_expr_kind_clone_drop_coverage() {
    let dummy_expr = Box::new(AnalyzedExpr {
        expr: AnalyzedExprKind::BooleanLiteral(true),
        glossa_type: GlossaType::Boolean,
    });
    let kinds = vec![
        AnalyzedExprKind::StringLiteral("test".to_string()),
        AnalyzedExprKind::NumberLiteral(10),
        AnalyzedExprKind::BooleanLiteral(true),
        AnalyzedExprKind::Variable(SmolStr::new("x")),
        AnalyzedExprKind::PropertyAccess {
            owner: dummy_expr.clone(),
            property: SmolStr::new("y"),
        },
        AnalyzedExprKind::VerbCall {
            verb: SmolStr::new("v"),
            args: vec![],
        },
        AnalyzedExprKind::FunctionCall {
            func: SmolStr::new("f"),
            args: vec![],
        },
        AnalyzedExprKind::MethodCall {
            receiver: dummy_expr.clone(),
            method: SmolStr::new("m"),
            args: vec![],
        },
        AnalyzedExprKind::BinOp {
            op: glossa::morphology::BinaryOp::Add,
            left: dummy_expr.clone(),
            right: dummy_expr.clone(),
        },
        AnalyzedExprKind::UnaryOp {
            op: glossa::morphology::UnaryOp::Not,
            operand: dummy_expr.clone(),
        },
        AnalyzedExprKind::ArrayLiteral(vec![]),
        AnalyzedExprKind::IndexAccess {
            array: dummy_expr.clone(),
            index: dummy_expr.clone(),
        },
        AnalyzedExprKind::Range {
            start: dummy_expr.clone(),
            end: dummy_expr.clone(),
            inclusive: true,
        },
        AnalyzedExprKind::StructInstantiation {
            type_name: SmolStr::new("t"),
            fields: vec![],
            args: vec![],
        },
        AnalyzedExprKind::Some(dummy_expr.clone()),
        AnalyzedExprKind::None,
        AnalyzedExprKind::Ok(dummy_expr.clone()),
        AnalyzedExprKind::Err(dummy_expr.clone()),
        AnalyzedExprKind::Try(dummy_expr.clone()),
        AnalyzedExprKind::Unwrap(dummy_expr.clone()),
        AnalyzedExprKind::CollectionNew {
            collection_type: "list".to_string(),
        },
        AnalyzedExprKind::Lambda {
            params: vec![],
            body: dummy_expr.clone(),
            capture_mode: CaptureMode::Move,
        },
        AnalyzedExprKind::Assert {
            condition: dummy_expr.clone(),
        },
        AnalyzedExprKind::AssertEq {
            left: dummy_expr.clone(),
            right: dummy_expr.clone(),
        },
    ];

    for kind in kinds {
        let cloned = kind.clone();
        drop(cloned);
        drop(kind);
    }
}

#[test]
fn test_analyzed_statement_clone_drop_coverage() {
    let dummy_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::BooleanLiteral(true),
        glossa_type: GlossaType::Boolean,
    };
    let stmts = vec![
        AnalyzedStatement::Binding {
            name: SmolStr::new("x"),
            value: dummy_expr.clone(),
            mutable: true,
        },
        AnalyzedStatement::Assignment {
            name: SmolStr::new("x"),
            value: dummy_expr.clone(),
        },
        AnalyzedStatement::Print(vec![dummy_expr.clone()]),
        AnalyzedStatement::Expression(vec![dummy_expr.clone()]),
        AnalyzedStatement::Query(vec![dummy_expr.clone()]),
        AnalyzedStatement::If {
            condition: Box::new(dummy_expr.clone()),
            then_body: vec![],
            else_body: None,
        },
        AnalyzedStatement::While {
            condition: Box::new(dummy_expr.clone()),
            body: vec![],
        },
        AnalyzedStatement::For {
            variable: SmolStr::new("v"),
            iterator: Box::new(dummy_expr.clone()),
            body: vec![],
        },
        AnalyzedStatement::Match {
            scrutinee: Box::new(dummy_expr.clone()),
            arms: vec![],
        },
        AnalyzedStatement::Break,
        AnalyzedStatement::Continue,
        AnalyzedStatement::Return { value: None },
        AnalyzedStatement::FunctionDef {
            name: SmolStr::new("f"),
            params: vec![],
            body: vec![],
            return_type: None,
        },
        AnalyzedStatement::TypeDefinition {
            name: SmolStr::new("T"),
            fields: vec![],
        },
        AnalyzedStatement::TraitDefinition {
            name: SmolStr::new("Tr"),
            methods: vec![],
        },
        AnalyzedStatement::TraitImplementation {
            trait_name: SmolStr::new("Tr"),
            type_name: SmolStr::new("T"),
            methods: vec![],
        },
        AnalyzedStatement::TestDeclaration {
            name: "test".to_string(),
            body: vec![],
        },
    ];

    for stmt in stmts {
        let cloned = stmt.clone();
        drop(cloned);
        drop(stmt);
    }
}
