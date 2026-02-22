use glossa::parser::parse;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, CaptureMode, GlossaType, analyze_program,
};
use glossa::tools::narrator::tell_tale;

fn compile_and_tell(source: &str) -> String {
    let ast = parse(source).expect("AST build failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    tell_tale(&analyzed)
}

#[test]
fn test_bard_binding_mutable() {
    let stmt = AnalyzedStatement::Binding {
        name: "x".into(),
        value: AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(5),
            glossa_type: GlossaType::Number,
        },
        mutable: true,
    };

    let program = glossa::semantic::AnalyzedProgram {
        statements: vec![stmt],
        scope: glossa::semantic::Scope::new(),
    };

    let tale = tell_tale(&program);
    assert!(tale.contains("BIND 📝"));
    assert!(tale.contains("Let"));
    assert!(tale.contains("x"));
    assert!(tale.contains("be"));
    assert!(tale.contains("5"));
    assert!(tale.contains("Mutable"));
}

#[test]
fn test_bard_assignment() {
    // Manually construct assignment to avoid mutable syntax issues in parser
    let stmt = AnalyzedStatement::Assignment {
        name: "x".into(),
        value: AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(10),
            glossa_type: GlossaType::Number,
        },
    };
    let program = glossa::semantic::AnalyzedProgram {
        statements: vec![stmt],
        scope: glossa::semantic::Scope::new(),
    };
    let tale = tell_tale(&program);
    assert!(tale.contains("SET ✏️"));
    assert!(tale.contains("Update"));
    assert!(tale.contains("x"));
    assert!(tale.contains("to"));
    assert!(tale.contains("10"));
}

#[test]
fn test_bard_print_multiple() {
    let source = "«α», «β» λέγε.";
    let tale = compile_and_tell(source);
    assert!(tale.contains("PRINT 📢"));
    assert!(tale.contains("Proclaim:"));
    assert!(tale.contains("α"));
    assert!(tale.contains("β"));
}

#[test]
fn test_bard_expression_stmt() {
    let stmt = AnalyzedStatement::Expression(vec![AnalyzedExpr {
        expr: AnalyzedExprKind::StringLiteral("test".into()),
        glossa_type: GlossaType::String,
    }]);
    let program = glossa::semantic::AnalyzedProgram {
        statements: vec![stmt],
        scope: glossa::semantic::Scope::new(),
    };
    let tale = tell_tale(&program);
    assert!(tale.contains("EXPR ⚡"));
    assert!(tale.contains("Do:"));
    assert!(tale.contains("test"));
}

#[test]
fn test_bard_query() {
    let source = "ξ πέντε ἔστω. ξ?";
    let tale = compile_and_tell(source);
    assert!(tale.contains("QUERY 🔮"));
    assert!(tale.contains("Query oracle:"));
    assert!(tale.contains("ξ"));
}

#[test]
fn test_bard_while() {
    let source = "ξ πέντε ἔστω. ἕως ξ μηδενὸς μεῖζον ᾖ, ξ λέγε.";
    let tale = compile_and_tell(source);
    assert!(tale.contains("WHILE 🔄"));
    assert!(tale.contains("While"));
    assert!(tale.contains("holds true"));
}

#[test]
fn test_bard_for() {
    let source = "ἀπὸ μηδενὸς μέχρι πέντε, ι λέγε.";
    let tale = compile_and_tell(source);
    assert!(tale.contains("FOR 🔁"));
    assert!(tale.contains("For each"));
    assert!(tale.contains("ι"));
    assert!(tale.contains("in"));
}

#[test]
fn test_bard_match() {
    let source = "ξ πέντε ἔστω. κατὰ ξ· μηδὲν ᾖ, «μηδέν» λέγε.";
    let tale = compile_and_tell(source);
    assert!(tale.contains("MATCH 🔍"));
    assert!(tale.contains("Match on"));
    assert!(tale.contains("ξ"));
    assert!(tale.contains("CASE 🎯"));
}

#[test]
fn test_bard_break_continue() {
    // Manually construct to ensure we hit the variants
    let stmts = vec![AnalyzedStatement::Break, AnalyzedStatement::Continue];
    let program = glossa::semantic::AnalyzedProgram {
        statements: stmts,
        scope: glossa::semantic::Scope::new(),
    };
    let tale = tell_tale(&program);
    assert!(tale.contains("BREAK 🛑"));
    assert!(tale.contains("Break loop"));
    assert!(tale.contains("CONT ⏩"));
    assert!(tale.contains("Continue loop"));
}

#[test]
fn test_bard_return() {
    let stmt = AnalyzedStatement::Return {
        value: Some(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(42),
            glossa_type: GlossaType::Number,
        })),
    };
    let program = glossa::semantic::AnalyzedProgram {
        statements: vec![stmt],
        scope: glossa::semantic::Scope::new(),
    };
    let tale = tell_tale(&program);
    assert!(tale.contains("RETURN 🚪"));
    assert!(tale.contains("Return"));
    assert!(tale.contains("42"));

    let stmt_empty = AnalyzedStatement::Return { value: None };
    let program_empty = glossa::semantic::AnalyzedProgram {
        statements: vec![stmt_empty],
        scope: glossa::semantic::Scope::new(),
    };
    let tale_empty = tell_tale(&program_empty);
    assert!(tale_empty.contains("Return nothing"));
}

#[test]
fn test_bard_function_def() {
    // Manually construct
    let stmt = AnalyzedStatement::FunctionDef {
        name: "my_func".into(),
        params: vec![("p1".into(), Some(GlossaType::Number))],
        body: vec![],
        return_type: Some(GlossaType::Boolean),
    };

    let program = glossa::semantic::AnalyzedProgram {
        statements: vec![stmt],
        scope: glossa::semantic::Scope::new(),
    };
    let tale = tell_tale(&program);
    assert!(tale.contains("FUNC ƒ"));
    assert!(tale.contains("Define"));
    assert!(tale.contains("my_func"));
    assert!(tale.contains("p1"));
    assert!(tale.contains("Number"));
    assert!(tale.contains("Bool"));
}

#[test]
fn test_bard_type_def() {
    let stmt = AnalyzedStatement::TypeDefinition {
        name: "MyType".into(),
        fields: vec![("field1".into(), GlossaType::Number)],
    };
    let program = glossa::semantic::AnalyzedProgram {
        statements: vec![stmt],
        scope: glossa::semantic::Scope::new(),
    };
    let tale = tell_tale(&program);
    assert!(tale.contains("TYPE 📦"));
    assert!(tale.contains("Struct"));
    assert!(tale.contains("MyType"));
    assert!(tale.contains("field1"));
    assert!(tale.contains("Number"));
}

#[test]
fn test_bard_trait_def() {
    let stmt = AnalyzedStatement::TraitDefinition {
        name: "MyTrait".into(),
        methods: vec![],
    };
    let program = glossa::semantic::AnalyzedProgram {
        statements: vec![stmt],
        scope: glossa::semantic::Scope::new(),
    };
    let tale = tell_tale(&program);
    assert!(tale.contains("TRAIT 🏷️"));
    assert!(tale.contains("Trait"));
    assert!(tale.contains("MyTrait"));
}

#[test]
fn test_bard_trait_impl() {
    let stmt = AnalyzedStatement::TraitImplementation {
        trait_name: "MyTrait".into(),
        type_name: "MyType".into(),
        methods: vec![],
    };
    let program = glossa::semantic::AnalyzedProgram {
        statements: vec![stmt],
        scope: glossa::semantic::Scope::new(),
    };
    let tale = tell_tale(&program);
    assert!(tale.contains("IMPL 🔧"));
    assert!(tale.contains("Impl"));
    assert!(tale.contains("MyTrait"));
    assert!(tale.contains("for"));
    assert!(tale.contains("MyType"));
}

#[test]
fn test_bard_test_decl() {
    let stmt = AnalyzedStatement::TestDeclaration {
        name: "my_test".into(),
        body: vec![],
    };
    let program = glossa::semantic::AnalyzedProgram {
        statements: vec![stmt],
        scope: glossa::semantic::Scope::new(),
    };
    let tale = tell_tale(&program);
    assert!(tale.contains("TEST 🧪"));
    assert!(tale.contains("Test"));
    assert!(tale.contains("my_test"));
}

// --- Expression Coverage ---

fn wrap_expr(expr: AnalyzedExprKind) -> AnalyzedStatement {
    AnalyzedStatement::Expression(vec![AnalyzedExpr {
        expr,
        glossa_type: GlossaType::Unknown,
    }])
}

fn test_expr_tale(kind: AnalyzedExprKind, parts: &[&str]) {
    let stmt = wrap_expr(kind);
    let program = glossa::semantic::AnalyzedProgram {
        statements: vec![stmt],
        scope: glossa::semantic::Scope::new(),
    };
    let tale = tell_tale(&program);
    for part in parts {
        assert!(tale.contains(part), "Expected '{}' in '{}'", part, tale);
    }
}

#[test]
fn test_bard_exprs() {
    test_expr_tale(AnalyzedExprKind::BooleanLiteral(true), &["true"]);
    test_expr_tale(
        AnalyzedExprKind::PropertyAccess {
            owner: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("obj".into()),
                glossa_type: GlossaType::Unknown,
            }),
            property: "prop".into(),
        },
        &["obj", "prop"],
    );

    test_expr_tale(
        AnalyzedExprKind::VerbCall {
            verb: "run".into(),
            args: vec![],
        },
        &["run"],
    );

    test_expr_tale(
        AnalyzedExprKind::UnaryOp {
            op: glossa::morphology::lexicon::UnaryOp::Not,
            operand: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: GlossaType::Boolean,
            }),
        },
        &["Not", "true"],
    );

    test_expr_tale(
        AnalyzedExprKind::Range {
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
        &["1", "..=", "10"],
    );

    test_expr_tale(AnalyzedExprKind::ArrayLiteral(vec![]), &["[]"]);

    test_expr_tale(
        AnalyzedExprKind::Some(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        })),
        &["Some", "1"],
    );
    test_expr_tale(AnalyzedExprKind::None, &["None"]);
    test_expr_tale(
        AnalyzedExprKind::Ok(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        })),
        &["Ok", "1"],
    );
    test_expr_tale(
        AnalyzedExprKind::Err(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        })),
        &["Err", "1"],
    );

    test_expr_tale(
        AnalyzedExprKind::Unwrap(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::Variable("x".into()),
            glossa_type: GlossaType::Option(Box::new(GlossaType::Number)),
        })),
        &["x", "!"],
    );
    test_expr_tale(
        AnalyzedExprKind::Try(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::Variable("x".into()),
            glossa_type: GlossaType::Option(Box::new(GlossaType::Number)),
        })),
        &["x", "?"],
    );

    test_expr_tale(
        AnalyzedExprKind::IndexAccess {
            array: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("arr".into()),
                glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
            }),
            index: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(0),
                glossa_type: GlossaType::Number,
            }),
        },
        &["arr", "[", "0", "]"],
    );

    test_expr_tale(
        AnalyzedExprKind::FunctionCall {
            func: "f".into(),
            args: vec![],
        },
        &["f", "(", ")"],
    );

    test_expr_tale(
        AnalyzedExprKind::MethodCall {
            receiver: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("obj".into()),
                glossa_type: GlossaType::Unknown,
            }),
            method: "m".into(),
            args: vec![],
        },
        &["obj", "m", "(", ")"],
    );

    test_expr_tale(
        AnalyzedExprKind::TraitMethodCall {
            receiver: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("obj".into()),
                glossa_type: GlossaType::Unknown,
            }),
            trait_name: "T".into(),
            method_name: "m".into(),
            args: vec![],
        },
        &["obj", "as", "T", "m"],
    );

    test_expr_tale(
        AnalyzedExprKind::StructInstantiation {
            type_name: "S".into(),
            fields: vec!["f".into()],
            args: vec![AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            }],
        },
        &["S", "f", "1"],
    );

    test_expr_tale(
        AnalyzedExprKind::Lambda {
            params: vec!["p".into()],
            body: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("p".into()),
                glossa_type: GlossaType::Unknown,
            }),
            capture_mode: CaptureMode::Borrow,
        },
        &["|", "p", "|", "p"],
    );

    test_expr_tale(
        AnalyzedExprKind::Lambda {
            params: vec![],
            body: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: GlossaType::Unit,
            }),
            capture_mode: CaptureMode::Move,
        },
        &["move", "||", "None"],
    );

    test_expr_tale(
        AnalyzedExprKind::Lambda {
            params: vec![],
            body: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: GlossaType::Unit,
            }),
            capture_mode: CaptureMode::Memoize,
        },
        &["memo", "||", "None"],
    );

    test_expr_tale(
        AnalyzedExprKind::CollectionNew {
            collection_type: "HashSet".into(),
        },
        &["HashSet", "new", "(", ")"],
    );

    test_expr_tale(
        AnalyzedExprKind::Assert {
            condition: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: GlossaType::Boolean,
            }),
        },
        &["assert", "true"],
    );

    test_expr_tale(
        AnalyzedExprKind::AssertEq {
            left: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            }),
            right: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            }),
        },
        &["assert_eq", "1", "1"],
    );
}

#[test]
fn test_bard_types() {
    fn check_type(ty: GlossaType, expected: &str) {
        let stmt = AnalyzedStatement::FunctionDef {
            name: "f".into(),
            params: vec![],
            body: vec![],
            return_type: Some(ty),
        };
        let program = glossa::semantic::AnalyzedProgram {
            statements: vec![stmt],
            scope: glossa::semantic::Scope::new(),
        };
        let tale = tell_tale(&program);
        assert!(
            tale.contains(expected),
            "Expected '{}' in '{}'",
            expected,
            tale
        );
    }

    check_type(GlossaType::List(Box::new(GlossaType::Number)), "[Number]");
    check_type(GlossaType::Set(Box::new(GlossaType::Number)), "Set<Number>");
    check_type(
        GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number)),
        "Map<String, Number>",
    );
    check_type(
        GlossaType::Option(Box::new(GlossaType::Number)),
        "Option<Number>",
    );
    check_type(
        GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String)),
        "Result<Number, String>",
    );
    check_type(
        GlossaType::Struct {
            name: "S".into(),
            gender: glossa::morphology::Gender::Neuter,
            fields: vec![],
        },
        "S",
    );
    check_type(
        GlossaType::Function {
            params: vec![GlossaType::Number],
            returns: Box::new(GlossaType::Boolean),
        },
        "Fn(Number) -> Bool",
    );
    check_type(GlossaType::Unit, "()");
    check_type(GlossaType::Unknown, "?");
}
