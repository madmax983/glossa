use glossa::tools::narrator::tell_tale;
use glossa::parser::parse;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, CaptureMode, GlossaType, analyze_program,
};

fn strip_ansi(s: &str) -> String {
    let mut output = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if let Some('[') = chars.peek() {
                chars.next(); // consume '['
                // consume until 'm'
                while let Some(c2) = chars.next() {
                    if c2 == 'm' {
                        break;
                    }
                }
            }
        } else {
            output.push(c);
        }
    }
    output
}

fn compile_and_tell(source: &str) -> String {
    let ast = parse(source).expect("AST build failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    strip_ansi(&tell_tale(&analyzed))
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

    let tale = strip_ansi(&tell_tale(&program));
    assert!(tale.contains("mutable variable named x"));
    assert!(tale.contains("with the value 5"));
}

#[test]
fn test_bard_assignment() {
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
    let tale = strip_ansi(&tell_tale(&program));
    assert!(tale.contains("Update x to become 10"));
}

#[test]
fn test_bard_print_multiple() {
    let source = "«α», «β» λέγε.";
    let tale = compile_and_tell(source);
    // "Proclaim to the world: "α", "β""
    assert!(tale.contains("Proclaim to the world"));
    assert!(tale.contains("\"α\""));
    assert!(tale.contains("\"β\""));
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
    let tale = strip_ansi(&tell_tale(&program));
    assert!(tale.contains("Perform the following: \"test\""));
}

#[test]
fn test_bard_query() {
    let source = "ξ πέντε ἔστω. ξ?";
    let tale = compile_and_tell(source);
    assert!(tale.contains("Query the oracle about: ξ"));
}

#[test]
fn test_bard_while() {
    let source = "ξ πέντε ἔστω. ἕως ξ μηδενὸς μεῖζον ᾖ, ξ λέγε.";
    let tale = compile_and_tell(source);
    assert!(tale.contains("As long as"));
    assert!(tale.contains("repeat:"));
}

#[test]
fn test_bard_for() {
    let source = "ἀπὸ μηδενὸς μέχρι πέντε, ι λέγε.";
    let tale = compile_and_tell(source);
    assert!(tale.contains("For each ι found in"));
}

#[test]
fn test_bard_match() {
    let source = "ξ πέντε ἔστω. κατὰ ξ· μηδὲν ᾖ, «μηδέν» λέγε.";
    let tale = compile_and_tell(source);
    assert!(tale.contains("Consider the nature of ξ"));
    assert!(tale.contains("In the case of"));
}

#[test]
fn test_bard_break_continue() {
    let stmts = vec![AnalyzedStatement::Break, AnalyzedStatement::Continue];
    let program = glossa::semantic::AnalyzedProgram {
        statements: stmts,
        scope: glossa::semantic::Scope::new(),
    };
    let tale = strip_ansi(&tell_tale(&program));
    assert!(tale.contains("Break the cycle"));
    assert!(tale.contains("Continue to the next iteration"));
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
    let tale = strip_ansi(&tell_tale(&program));
    assert!(tale.contains("Return with the offering 42"));

    let stmt_empty = AnalyzedStatement::Return { value: None };
    let program_empty = glossa::semantic::AnalyzedProgram {
        statements: vec![stmt_empty],
        scope: glossa::semantic::Scope::new(),
    };
    let tale_empty = strip_ansi(&tell_tale(&program_empty));
    assert!(tale_empty.contains("Return with nothing"));
}

#[test]
fn test_bard_function_def() {
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
    let tale = strip_ansi(&tell_tale(&program));
    assert!(tale.contains("Define a ritual called my_func"));
    assert!(tale.contains("expecting [p1 (Number)]"));
    assert!(tale.contains("returns Truth"));
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
    let tale = strip_ansi(&tell_tale(&program));
    assert!(tale.contains("Declare a new form MyType"));
    assert!(tale.contains("attributes: field1 as Number"));
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
    let tale = strip_ansi(&tell_tale(&program));
    assert!(tale.contains("Declare a characteristic named MyTrait"));
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
    let tale = strip_ansi(&tell_tale(&program));
    assert!(tale.contains("Imbue MyType with the characteristic of MyTrait"));
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
    let tale = strip_ansi(&tell_tale(&program));
    assert!(tale.contains("Define a trial named my_test"));
}

// --- Expression Coverage ---

fn wrap_expr(expr: AnalyzedExprKind) -> AnalyzedStatement {
    AnalyzedStatement::Expression(vec![AnalyzedExpr {
        expr,
        glossa_type: GlossaType::Unknown,
    }])
}

fn test_expr_tale(kind: AnalyzedExprKind, expected: &str) {
    let stmt = wrap_expr(kind);
    let program = glossa::semantic::AnalyzedProgram {
        statements: vec![stmt],
        scope: glossa::semantic::Scope::new(),
    };
    let tale = strip_ansi(&tell_tale(&program));
    assert!(
        tale.contains(expected),
        "Expected '{}' in '{}'",
        expected,
        tale
    );
}

#[test]
fn test_bard_exprs() {
    test_expr_tale(AnalyzedExprKind::BooleanLiteral(true), "true");
    test_expr_tale(
        AnalyzedExprKind::PropertyAccess {
            owner: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("obj".into()),
                glossa_type: GlossaType::Unknown,
            }),
            property: "prop".into(),
        },
        "the prop of obj",
    );

    test_expr_tale(
        AnalyzedExprKind::VerbCall {
            verb: "run".into(),
            args: vec![],
        },
        "runing []",
    );

    test_expr_tale(
        AnalyzedExprKind::UnaryOp {
            op: glossa::morphology::lexicon::UnaryOp::Not,
            operand: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: GlossaType::Boolean,
            }),
        },
        "(Not true)",
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
        "from 1 through 10",
    );

    test_expr_tale(
        AnalyzedExprKind::ArrayLiteral(vec![]),
        "[]",
    );

    test_expr_tale(
        AnalyzedExprKind::Some(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        })),
        "Some(1)",
    );
    test_expr_tale(AnalyzedExprKind::None, "None");
    test_expr_tale(
        AnalyzedExprKind::Ok(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        })),
        "Ok(1)",
    );
    test_expr_tale(
        AnalyzedExprKind::Err(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        })),
        "Err(1)",
    );

    test_expr_tale(
        AnalyzedExprKind::Unwrap(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::Variable("x".into()),
            glossa_type: GlossaType::Option(Box::new(GlossaType::Number)),
        })),
        "unwrap(x)",
    );
    test_expr_tale(
        AnalyzedExprKind::Try(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::Variable("x".into()),
            glossa_type: GlossaType::Option(Box::new(GlossaType::Number)),
        })),
        "try(x)",
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
        "arr at index 0",
    );

    test_expr_tale(
        AnalyzedExprKind::FunctionCall {
            func: "f".into(),
            args: vec![],
        },
        "call f()",
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
        "obj.m()",
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
        "<T>.m(obj, )",
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
        "S { f: 1 }",
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
        "|p| p",
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
        "move || None",
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
        "memo || None",
    );

    test_expr_tale(
        AnalyzedExprKind::CollectionNew {
            collection_type: "HashSet".into(),
        },
        "new HashSet",
    );

    test_expr_tale(
        AnalyzedExprKind::Assert {
            condition: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: GlossaType::Boolean,
            }),
        },
        "assert(true)",
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
        "assert_eq(1, 1)",
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
        let tale = strip_ansi(&tell_tale(&program));
        assert!(
            tale.contains(expected),
            "Expected '{}' in '{}'",
            expected,
            tale
        );
    }

    check_type(
        GlossaType::List(Box::new(GlossaType::Number)),
        "List<Number>",
    );
    check_type(
        GlossaType::Set(Box::new(GlossaType::Number)),
        "Set<Number>",
    );
    check_type(
        GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number)),
        "Map<Text, Number>",
    );
    check_type(
        GlossaType::Option(Box::new(GlossaType::Number)),
        "Option<Number>",
    );
    check_type(
        GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String)),
        "Result<Number, Text>",
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
        "fn(Number) -> Truth",
    );
    check_type(GlossaType::Unit, "()");
    check_type(GlossaType::Unknown, "?");
}
