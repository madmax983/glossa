#![cfg(feature = "nova")]

use glossa::morphology::lexicon::{BinaryOp, UnaryOp};
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};
use glossa::tools::alchemist::transpile_to_python;

#[test]
fn test_alchemist_coverage() {
    let statements = vec![
        // While loop
        AnalyzedStatement::While {
            condition: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: GlossaType::Boolean,
            }),
            body: vec![AnalyzedStatement::Break, AnalyzedStatement::Continue],
        },
        // While empty body
        AnalyzedStatement::While {
            condition: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(false),
                glossa_type: GlossaType::Boolean,
            }),
            body: vec![],
        },
        // For loop
        AnalyzedStatement::For {
            variable: "x".into(),
            iterator: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Range {
                    start: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                    end: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(10),
                        glossa_type: GlossaType::Number,
                    }),
                    inclusive: true,
                },
                glossa_type: GlossaType::Number,
            }),
            body: vec![AnalyzedStatement::Return {
                value: Some(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(42),
                    glossa_type: GlossaType::Number,
                })),
            }],
        },
        // For empty body
        AnalyzedStatement::For {
            variable: "y".into(),
            iterator: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::ArrayLiteral(vec![AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }]),
                glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
            }),
            body: vec![],
        },
        // Return none
        AnalyzedStatement::Return { value: None },
        // Function with empty body
        AnalyzedStatement::FunctionDef {
            name: "empty_func".into(),
            params: vec![],
            body: vec![],
            return_type: None,
        },
        // Type Definition
        AnalyzedStatement::TypeDefinition {
            name: "User".into(),
            fields: vec![("name".into(), GlossaType::String)],
        },
        AnalyzedStatement::TypeDefinition {
            name: "Empty".into(),
            fields: vec![],
        },
        // Test Declaration
        AnalyzedStatement::TestDeclaration {
            name: "test feature".into(),
            body: vec![AnalyzedStatement::Print(vec![AnalyzedExpr {
                expr: AnalyzedExprKind::StringLiteral("test".into()),
                glossa_type: GlossaType::String,
            }])],
        },
        AnalyzedStatement::TestDeclaration {
            name: "test_empty".into(),
            body: vec![],
        },
        // Match Statement
        AnalyzedStatement::Match {
            scrutinee: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            }),
            arms: vec![
                (
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    },
                    vec![AnalyzedStatement::Break],
                ),
                (
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable("x".into()),
                        glossa_type: GlossaType::Number,
                    },
                    vec![],
                ),
            ],
        },
        // Expressions
        AnalyzedStatement::Expression(vec![
            AnalyzedExpr {
                expr: AnalyzedExprKind::PropertyAccess {
                    owner: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable("obj".into()),
                        glossa_type: GlossaType::Unknown,
                    }),
                    property: "prop".into(),
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::VerbCall {
                    verb: "action".into(),
                    args: vec![],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::FunctionCall {
                    func: "func".into(),
                    args: vec![],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::StructInstantiation {
                    type_name: "User".into(),
                    fields: vec!["name".into()],
                    args: vec![AnalyzedExpr {
                        expr: AnalyzedExprKind::StringLiteral("Jules".into()),
                        glossa_type: GlossaType::String,
                    }],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::UnaryOp {
                    op: UnaryOp::Ref,
                    operand: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable("x".into()),
                        glossa_type: GlossaType::Unknown,
                    }),
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::BinOp {
                    left: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                    op: BinaryOp::Mod,
                    right: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(2),
                        glossa_type: GlossaType::Number,
                    }),
                },
                glossa_type: GlossaType::Number,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::BinOp {
                    left: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                    op: BinaryOp::Le,
                    right: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(2),
                        glossa_type: GlossaType::Number,
                    }),
                },
                glossa_type: GlossaType::Boolean,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::BinOp {
                    left: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                    op: BinaryOp::Ge,
                    right: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(2),
                        glossa_type: GlossaType::Number,
                    }),
                },
                glossa_type: GlossaType::Boolean,
            },
            // Fallback unhandled expr
            AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: GlossaType::Unknown,
            },
        ]),
        // Fallback unhandled stmt
        AnalyzedStatement::TraitDefinition {
            name: "T".into(),
            methods: vec![],
        },
    ];

    let program = AnalyzedProgram {
        statements,
        scope: Scope::new(),
    };

    let py = transpile_to_python(&program);

    println!("--- Python Coverage ---\n{}", py);

    // Verify some outputs to ensure things generated
    assert!(py.contains("while True:"));
    assert!(py.contains("break"));
    assert!(py.contains("continue"));
    assert!(py.contains("for g_x in range(1, 10 + 1):"));
    assert!(py.contains("return 42"));
    assert!(py.contains("for g_y in [1]:"));
    assert!(py.contains("return"));
    assert!(py.contains("def g_empty_func():\n    pass"));
    assert!(py.contains("@dataclass\nclass g_User:"));
    assert!(py.contains("g_name: Any"));
    assert!(py.contains("def test_test_feature():"));
    assert!(py.contains("match 1:"));
    assert!(py.contains("case 1:"));
    assert!(py.contains("g_obj.g_prop"));
    assert!(py.contains("g_action()"));
    assert!(py.contains("g_func()"));
    assert!(py.contains("g_User(g_name=\"Jules\")"));
    assert!(py.contains("(1 % 2)"));
    assert!(py.contains("(1 <= 2)"));
    assert!(py.contains("(1 >= 2)"));
}
