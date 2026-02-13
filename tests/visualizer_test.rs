use glossa::experimental::visualizer::Visualizer;
use glossa::parser::parse;
use glossa::semantic::analyze_program;
use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope};
use glossa::semantic::AnalyzedProgram;

#[test]
fn test_visualize_if_else() {
    let source = r#"
        ξ πέντε ἔστω.
        εἰ ξ μεῖζον πέντε,
            «Μεγάλο» λέγε · εἰ δὲ μή,
            «Μικρό» λέγε.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("flowchart TD"));
    assert!(mermaid.contains("If"));
    assert!(mermaid.contains("Then"));
    assert!(mermaid.contains("Else"));
    assert!(mermaid.contains("Print: «Μεγάλο»"));
    assert!(mermaid.contains("Print: «Μικρό»"));
    assert!(mermaid.contains("-->"));
}

#[test]
fn test_visualize_loop() {
    let source = r#"
        ξ μηδέν ἔστω.
        ἕως ἀληθές,
            «Loop» λέγε.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("While true"));
    assert!(mermaid.contains(":::loop"));
}

#[test]
fn test_visualize_function() {
    let source = r#"
        test_func ὁρίζειν (x)·
            x λέγε.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("subgraph test_func"));
    assert!(mermaid.contains("([fn test_func])"));
}

#[test]
fn test_visualize_assignment() {
    let source = r#"
        μετὰ ξ πέντε ἔστω.
        ξ δέκα γίγνεται.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("let ξ = 5"));
    assert!(mermaid.contains("ξ = 10"));
}

#[test]
fn test_visualize_expression_stmts() {
    let source = r#"
        1 2 ἄθροισμα.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("1"));
    assert!(mermaid.contains("2"));
    assert!(mermaid.contains("Add"));
}

#[test]
fn test_visualize_match() {
    let source = r#"
        ξ πέντε ἔστω.
        κατὰ ξ·
        πέντε ᾖ, «Five» λέγε·
        ἄλλο ᾖ, «Other» λέγε.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("Match ξ"));
    assert!(mermaid.contains("Case: 5"));
    assert!(mermaid.contains("Case: true"));
    assert!(mermaid.contains(":::cond"));
}

#[test]
fn test_visualize_for_loop() {
    let source = r#"
        ξ μηδέν ἔστω.
        μετὰ ι μηδέν ἔστω.
        ἀπὸ ι μέχρι δέκα,
            «Iter» λέγε.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("For i in ι..10"));
    assert!(mermaid.contains(":::loop"));
}

#[test]
fn test_visualize_return_break_continue() {
    let source = r#"
        δός 42.
        παῦε.
        συνέχιζε.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("Break"));
    assert!(mermaid.contains("Continue"));
    assert!(mermaid.contains("Return 42"));
}

#[test]
fn test_visualize_query() {
    let source = r#"
        ξ πέντε ἔστω.
        ξ?
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("Query: ξ"));
    assert!(mermaid.contains(":::io"));
}

#[test]
fn test_visualize_unwrap_try() {
    let stmt = AnalyzedStatement::Expression(vec![
        AnalyzedExpr {
            expr: AnalyzedExprKind::Unwrap(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("x".into()),
                glossa_type: GlossaType::Unknown,
            })),
            glossa_type: GlossaType::Unknown,
        },
        AnalyzedExpr {
            expr: AnalyzedExprKind::Try(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("y".into()),
                glossa_type: GlossaType::Unknown,
            })),
            glossa_type: GlossaType::Unknown,
        },
    ]);

    let program = AnalyzedProgram {
        statements: vec![stmt],
        scope: Scope::new(),
    };

    let mut visualizer = Visualizer::new(&program);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("x!"));
    assert!(mermaid.contains("y?"));
}

#[test]
fn test_visualize_array_index() {
    let stmt = AnalyzedStatement::Expression(vec![
        AnalyzedExpr {
            expr: AnalyzedExprKind::IndexAccess {
                array: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("arr".into()),
                    glossa_type: GlossaType::Unknown,
                }),
                index: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(0),
                    glossa_type: GlossaType::Number,
                }),
            },
            glossa_type: GlossaType::Unknown,
        },
        AnalyzedExpr {
            expr: AnalyzedExprKind::ArrayLiteral(vec![AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            }]),
            glossa_type: GlossaType::Unknown,
        },
    ]);

    let program = AnalyzedProgram {
        statements: vec![stmt],
        scope: Scope::new(),
    };

    let mut visualizer = Visualizer::new(&program);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("arr(0)"));
    assert!(mermaid.contains("(1)"));
}

#[test]
fn test_visualize_struct_new() {
    let stmt = AnalyzedStatement::Expression(vec![AnalyzedExpr {
        expr: AnalyzedExprKind::StructInstantiation {
            type_name: "Point".into(),
            fields: vec!["x".into(), "y".into()],
            args: vec![],
        },
        glossa_type: GlossaType::Unknown,
    }]);

    let type_def = AnalyzedStatement::TypeDefinition {
        name: "Point".into(),
        fields: vec![],
    };

    let program = AnalyzedProgram {
        statements: vec![type_def, stmt],
        scope: Scope::new(),
    };

    let mut visualizer = Visualizer::new(&program);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("Type Point"));
    assert!(mermaid.contains("new Point"));
}

#[test]
fn test_visualize_asserts() {
    let stmt = AnalyzedStatement::Expression(vec![
        AnalyzedExpr {
            expr: AnalyzedExprKind::Assert {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
            },
            glossa_type: GlossaType::Unit,
        },
        AnalyzedExpr {
            expr: AnalyzedExprKind::AssertEq {
                left: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
                right: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
            },
            glossa_type: GlossaType::Unit,
        },
    ]);

    let program = AnalyzedProgram {
        statements: vec![stmt],
        scope: Scope::new(),
    };

    let mut visualizer = Visualizer::new(&program);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("Assert(true)"));
    assert!(mermaid.contains("AssertEq(1, 1)"));
}

#[test]
fn test_visualize_extended_exprs() {
    // Tests for PropertyAccess, UnaryOp, Option/Result, Lambda, CollectionNew
    // ADDED: Ok, Err, TraitMethodCall
    let stmt = AnalyzedStatement::Expression(vec![
        // PropertyAccess
        AnalyzedExpr {
            expr: AnalyzedExprKind::PropertyAccess {
                owner: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("x".into()),
                    glossa_type: GlossaType::Unknown,
                }),
                property: "len".into(),
            },
            glossa_type: GlossaType::Number,
        },
        // UnaryOp
        AnalyzedExpr {
            expr: AnalyzedExprKind::UnaryOp {
                op: glossa::morphology::UnaryOp::Not,
                operand: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
            },
            glossa_type: GlossaType::Boolean,
        },
        // Some
        AnalyzedExpr {
            expr: AnalyzedExprKind::Some(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            })),
            glossa_type: GlossaType::Option(Box::new(GlossaType::Number)),
        },
        // Ok
        AnalyzedExpr {
            expr: AnalyzedExprKind::Ok(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(2),
                glossa_type: GlossaType::Number,
            })),
            glossa_type: GlossaType::Result(
                Box::new(GlossaType::Number),
                Box::new(GlossaType::String),
            ),
        },
        // Err
        AnalyzedExpr {
            expr: AnalyzedExprKind::Err(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::StringLiteral("e".into()),
                glossa_type: GlossaType::String,
            })),
            glossa_type: GlossaType::Result(
                Box::new(GlossaType::Number),
                Box::new(GlossaType::String),
            ),
        },
        // None
        AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: GlossaType::Option(Box::new(GlossaType::Number)),
        },
        // Lambda
        AnalyzedExpr {
            expr: AnalyzedExprKind::Lambda {
                params: vec![],
                body: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
                capture_mode: glossa::semantic::CaptureMode::Borrow,
            },
            glossa_type: GlossaType::Unknown,
        },
        // CollectionNew
        AnalyzedExpr {
            expr: AnalyzedExprKind::CollectionNew {
                collection_type: "HashSet".into(),
            },
            glossa_type: GlossaType::Unknown,
        },
        // TraitMethodCall
        AnalyzedExpr {
            expr: AnalyzedExprKind::TraitMethodCall {
                receiver: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("x".into()),
                    glossa_type: GlossaType::Unknown,
                }),
                trait_name: "Show".into(),
                method_name: "print".into(),
                args: vec![],
            },
            glossa_type: GlossaType::Unit,
        },
    ]);

    let program = AnalyzedProgram {
        statements: vec![stmt],
        scope: Scope::new(),
    };

    let mut visualizer = Visualizer::new(&program);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("x.len"));
    assert!(mermaid.contains("Not true"));
    assert!(mermaid.contains("Some(1)"));
    assert!(mermaid.contains("Ok(2)"));
    assert!(mermaid.contains("Err(«e»)"));
    assert!(mermaid.contains("None"));
    assert!(mermaid.contains("Lambda"));
    assert!(mermaid.contains("new HashSet"));
    assert!(mermaid.contains("Show::print(...)"));
}

#[test]
fn test_visualize_definitions() {
    // TraitDefinition, TraitImplementation, TestDeclaration
    let trait_def = AnalyzedStatement::TraitDefinition {
        name: "Show".into(),
        methods: vec![],
    };
    let trait_impl = AnalyzedStatement::TraitImplementation {
        trait_name: "Show".into(),
        type_name: "String".into(),
        methods: vec![],
    };
    let test_decl = AnalyzedStatement::TestDeclaration {
        name: "my_test".into(),
        body: vec![],
    };

    let program = AnalyzedProgram {
        statements: vec![trait_def, trait_impl, test_decl],
        scope: Scope::new(),
    };

    let mut visualizer = Visualizer::new(&program);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("Trait Show"));
    assert!(mermaid.contains("Impl Show for String"));
    assert!(mermaid.contains("subgraph Test_my_test"));
}
