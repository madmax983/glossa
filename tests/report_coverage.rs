use glossa::report::ProgramStats;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, CaptureMode, GlossaType,
    Scope,
};

fn expr(kind: AnalyzedExprKind) -> AnalyzedExpr {
    AnalyzedExpr {
        expr: kind,
        glossa_type: GlossaType::Unknown,
    }
}

#[test]
fn test_report_manual_ast_coverage() {
    let statements = vec![
        // 1. Binding with BinOp and UnaryOp
        AnalyzedStatement::Binding {
            name: "x".into(),
            value: expr(AnalyzedExprKind::BinOp {
                left: Box::new(expr(AnalyzedExprKind::UnaryOp {
                    op: glossa::morphology::lexicon::UnaryOp::Neg,
                    operand: Box::new(expr(AnalyzedExprKind::NumberLiteral(1))),
                })),
                op: glossa::morphology::lexicon::BinaryOp::Add,
                right: Box::new(expr(AnalyzedExprKind::NumberLiteral(2))),
            }),
            mutable: false,
        },
        // 2. Assignment with FunctionCall and StructInstantiation
        AnalyzedStatement::Assignment {
            name: "x".into(),
            value: expr(AnalyzedExprKind::FunctionCall {
                func: "my_func".into(),
                args: vec![
                    expr(AnalyzedExprKind::StringLiteral("arg".into())),
                    expr(AnalyzedExprKind::StructInstantiation {
                        type_name: "Point".into(),
                        fields: vec!["x".into(), "y".into()],
                        args: vec![
                            expr(AnalyzedExprKind::NumberLiteral(0)),
                            expr(AnalyzedExprKind::NumberLiteral(0)),
                        ],
                    }),
                ],
            }),
        },
        // 3. Print with PropertyAccess, TraitMethodCall, VerbCall and other exprs
        AnalyzedStatement::Print(vec![
            expr(AnalyzedExprKind::PropertyAccess {
                owner: Box::new(expr(AnalyzedExprKind::Variable("obj".into()))),
                property: "prop".into(),
            }),
            expr(AnalyzedExprKind::ArrayLiteral(vec![])),
            expr(AnalyzedExprKind::Unwrap(Box::new(expr(
                AnalyzedExprKind::Variable("x".into()),
            )))),
            expr(AnalyzedExprKind::IndexAccess {
                array: Box::new(expr(AnalyzedExprKind::Variable("arr".into()))),
                index: Box::new(expr(AnalyzedExprKind::NumberLiteral(0))),
            }),
            expr(AnalyzedExprKind::MethodCall {
                receiver: Box::new(expr(AnalyzedExprKind::Variable("obj".into()))),
                method: "meth".into(),
                args: vec![],
            }),
            expr(AnalyzedExprKind::TraitMethodCall {
                receiver: Box::new(expr(AnalyzedExprKind::Variable("obj".into()))),
                trait_name: "Trait".into(),
                method_name: "meth".into(),
                args: vec![],
            }),
            expr(AnalyzedExprKind::VerbCall {
                verb: "run".into(),
                args: vec![expr(AnalyzedExprKind::Variable("x".into()))],
            }),
            expr(AnalyzedExprKind::Lambda {
                params: vec!["p".into()],
                body: Box::new(expr(AnalyzedExprKind::NumberLiteral(1))),
                capture_mode: CaptureMode::Borrow,
            }),
        ]),
        // 4. If with Block, AssertEq and various result types (Some, Ok, Try)
        AnalyzedStatement::If {
            condition: Box::new(expr(AnalyzedExprKind::AssertEq {
                left: Box::new(expr(AnalyzedExprKind::BooleanLiteral(true))),
                right: Box::new(expr(AnalyzedExprKind::BooleanLiteral(true))),
            })),
            then_body: vec![AnalyzedStatement::Expression(vec![
                expr(AnalyzedExprKind::None),
                expr(AnalyzedExprKind::Try(Box::new(expr(
                    AnalyzedExprKind::Variable("r".into()),
                )))),
            ])],
            else_body: Some(vec![AnalyzedStatement::Return {
                value: Some(Box::new(expr(AnalyzedExprKind::Ok(Box::new(expr(
                    AnalyzedExprKind::Some(Box::new(expr(AnalyzedExprKind::NumberLiteral(42)))),
                )))))),
            }]),
        },
        // 5. While Loop with Break/Continue/Err
        AnalyzedStatement::While {
            condition: Box::new(expr(AnalyzedExprKind::BooleanLiteral(true))),
            body: vec![
                AnalyzedStatement::Break,
                AnalyzedStatement::Continue,
                AnalyzedStatement::Expression(vec![expr(AnalyzedExprKind::Err(Box::new(expr(
                    AnalyzedExprKind::StringLiteral("error".into()),
                ))))]),
            ],
        },
        // 6. For Loop with Range and Query
        AnalyzedStatement::For {
            variable: "i".into(),
            iterator: Box::new(expr(AnalyzedExprKind::Range {
                start: Box::new(expr(AnalyzedExprKind::NumberLiteral(0))),
                end: Box::new(expr(AnalyzedExprKind::NumberLiteral(10))),
                inclusive: false,
            })),
            body: vec![AnalyzedStatement::Query(vec![expr(
                AnalyzedExprKind::Variable("i".into()),
            )])],
        },
        // 7. Match
        AnalyzedStatement::Match {
            scrutinee: Box::new(expr(AnalyzedExprKind::Variable("x".into()))),
            arms: vec![(
                expr(AnalyzedExprKind::NumberLiteral(1)),
                vec![AnalyzedStatement::Expression(vec![expr(
                    AnalyzedExprKind::Variable("y".into()),
                )])],
            )],
        },
        // 8. Test Declaration with Assert
        AnalyzedStatement::TestDeclaration {
            name: "test".into(),
            body: vec![AnalyzedStatement::Expression(vec![expr(
                AnalyzedExprKind::Assert {
                    condition: Box::new(expr(AnalyzedExprKind::BooleanLiteral(true))),
                },
            )])],
        },
        // 9. Function Definition
        AnalyzedStatement::FunctionDef {
            name: "func".into(),
            params: vec![],
            body: vec![AnalyzedStatement::Return { value: None }],
            return_type: None,
        },
        // 10. Type/Trait Definitions (to cover remaining empty branches)
        AnalyzedStatement::TypeDefinition {
            name: "MyType".into(),
            fields: vec![],
        },
        AnalyzedStatement::TraitDefinition {
            name: "MyTrait".into(),
            methods: vec![],
        },
        AnalyzedStatement::TraitImplementation {
            trait_name: "MyTrait".into(),
            type_name: "MyType".into(),
            methods: vec![],
        },
    ];

    // Construct program
    let program = AnalyzedProgram {
        statements,
        scope: Scope::new(),
    };

    // Run stats generation (visitor)
    let stats = ProgramStats::new(&program);

    // Verify stats
    assert!(stats.statement_count >= 12);
    assert!(stats.expression_count > 25);
    assert_eq!(stats.binding_count, 1);
    assert_eq!(stats.loop_count, 2);
    assert_eq!(stats.conditional_count, 2);
}
