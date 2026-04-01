use glossa::ast::*;

#[test]
fn test_expr_clone() {
    let w = Word::new("λέγε");

    let all_exprs = vec![
        Expr::StringLiteral("test".to_string()),
        Expr::NumberLiteral(42),
        Expr::BooleanLiteral(true),
        Expr::ArrayLiteral(vec![Expr::NumberLiteral(1)]),
        Expr::IndexAccess {
            array: Box::new(Expr::ArrayLiteral(vec![Expr::NumberLiteral(2)])),
            index: Box::new(Expr::NumberLiteral(0)),
        },
        Expr::Word(w.clone()),
        Expr::Phrase(vec![Expr::NumberLiteral(3)]),
        Expr::PropertyAccess {
            owner: Box::new(Expr::Word(w.clone())),
            property: Box::new(Expr::Word(w.clone())),
        },
        Expr::Call {
            verb: w.clone(),
            arguments: vec![Expr::NumberLiteral(4)],
        },
        Expr::Binding {
            name: w.clone(),
            value: Box::new(Expr::NumberLiteral(5)),
        },
        Expr::BinOp {
            left: Box::new(Expr::NumberLiteral(6)),
            op: BinOperator::Add,
            right: Box::new(Expr::NumberLiteral(7)),
        },
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            operand: Box::new(Expr::BooleanLiteral(false)),
        },
        Expr::Block(vec![Statement::Regular {
            clauses: vec![],
            is_query: false,
            is_propagate: false,
        }]),
    ];

    for expr in &all_exprs {
        let cloned = expr.clone();
        assert_eq!(expr, &cloned);
    }
}

#[test]
fn test_expr_eq_different_types() {
    let w = Word::new("λέγε");

    let all_exprs = vec![
        Expr::StringLiteral("test".to_string()),
        Expr::NumberLiteral(42),
        Expr::BooleanLiteral(true),
        Expr::ArrayLiteral(vec![Expr::NumberLiteral(1)]),
        Expr::IndexAccess {
            array: Box::new(Expr::ArrayLiteral(vec![Expr::NumberLiteral(2)])),
            index: Box::new(Expr::NumberLiteral(0)),
        },
        Expr::Word(w.clone()),
        Expr::Phrase(vec![Expr::NumberLiteral(3)]),
        Expr::PropertyAccess {
            owner: Box::new(Expr::Word(w.clone())),
            property: Box::new(Expr::Word(w.clone())),
        },
        Expr::Call {
            verb: w.clone(),
            arguments: vec![Expr::NumberLiteral(4)],
        },
        Expr::Binding {
            name: w.clone(),
            value: Box::new(Expr::NumberLiteral(5)),
        },
        Expr::BinOp {
            left: Box::new(Expr::NumberLiteral(6)),
            op: BinOperator::Add,
            right: Box::new(Expr::NumberLiteral(7)),
        },
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            operand: Box::new(Expr::BooleanLiteral(false)),
        },
        Expr::Block(vec![Statement::Regular {
            clauses: vec![],
            is_query: false,
            is_propagate: false,
        }]),
    ];

    for i in 0..all_exprs.len() {
        for j in 0..all_exprs.len() {
            if i != j {
                assert_ne!(&all_exprs[i], &all_exprs[j]);
            }
        }
    }
}

#[test]
fn test_expr_eq_same_variants_different_data() {
    assert_ne!(
        Expr::StringLiteral("a".to_string()),
        Expr::StringLiteral("b".to_string())
    );
    assert_ne!(Expr::NumberLiteral(1), Expr::NumberLiteral(2));
    assert_ne!(Expr::BooleanLiteral(true), Expr::BooleanLiteral(false));
    assert_ne!(
        Expr::ArrayLiteral(vec![Expr::NumberLiteral(1)]),
        Expr::ArrayLiteral(vec![Expr::NumberLiteral(2)])
    );
    assert_ne!(
        Expr::IndexAccess {
            array: Box::new(Expr::ArrayLiteral(vec![Expr::NumberLiteral(1)])),
            index: Box::new(Expr::NumberLiteral(0)),
        },
        Expr::IndexAccess {
            array: Box::new(Expr::ArrayLiteral(vec![Expr::NumberLiteral(2)])),
            index: Box::new(Expr::NumberLiteral(0)),
        }
    );
    assert_ne!(Expr::Word(Word::new("α")), Expr::Word(Word::new("β")));
    assert_ne!(
        Expr::Phrase(vec![Expr::NumberLiteral(1)]),
        Expr::Phrase(vec![Expr::NumberLiteral(2)])
    );
    assert_ne!(
        Expr::PropertyAccess {
            owner: Box::new(Expr::Word(Word::new("α"))),
            property: Box::new(Expr::Word(Word::new("β"))),
        },
        Expr::PropertyAccess {
            owner: Box::new(Expr::Word(Word::new("γ"))),
            property: Box::new(Expr::Word(Word::new("δ"))),
        }
    );
    assert_ne!(
        Expr::Call {
            verb: Word::new("α"),
            arguments: vec![Expr::NumberLiteral(1)],
        },
        Expr::Call {
            verb: Word::new("β"),
            arguments: vec![Expr::NumberLiteral(2)],
        }
    );
    assert_ne!(
        Expr::Binding {
            name: Word::new("α"),
            value: Box::new(Expr::NumberLiteral(1)),
        },
        Expr::Binding {
            name: Word::new("β"),
            value: Box::new(Expr::NumberLiteral(2)),
        }
    );
    assert_ne!(
        Expr::BinOp {
            left: Box::new(Expr::NumberLiteral(1)),
            op: BinOperator::Add,
            right: Box::new(Expr::NumberLiteral(2)),
        },
        Expr::BinOp {
            left: Box::new(Expr::NumberLiteral(3)),
            op: BinOperator::Sub,
            right: Box::new(Expr::NumberLiteral(4)),
        }
    );
    assert_ne!(
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            operand: Box::new(Expr::BooleanLiteral(true)),
        },
        Expr::UnaryOp {
            op: UnaryOperator::Neg,
            operand: Box::new(Expr::BooleanLiteral(false)),
        }
    );
    assert_ne!(
        Expr::Block(vec![]),
        Expr::Block(vec![Statement::Regular {
            clauses: vec![],
            is_query: false,
            is_propagate: false,
        }])
    );
}

#[test]
fn test_expr_drop() {
    let w = Word::new("λέγε");

    let all_exprs = vec![
        Expr::StringLiteral("test".to_string()),
        Expr::NumberLiteral(42),
        Expr::BooleanLiteral(true),
        Expr::ArrayLiteral(vec![Expr::NumberLiteral(1)]),
        Expr::IndexAccess {
            array: Box::new(Expr::ArrayLiteral(vec![Expr::NumberLiteral(2)])),
            index: Box::new(Expr::NumberLiteral(0)),
        },
        Expr::Word(w.clone()),
        Expr::Phrase(vec![Expr::NumberLiteral(3)]),
        Expr::PropertyAccess {
            owner: Box::new(Expr::Word(w.clone())),
            property: Box::new(Expr::Word(w.clone())),
        },
        Expr::Call {
            verb: w.clone(),
            arguments: vec![Expr::NumberLiteral(4)],
        },
        Expr::Binding {
            name: w.clone(),
            value: Box::new(Expr::NumberLiteral(5)),
        },
        Expr::BinOp {
            left: Box::new(Expr::NumberLiteral(6)),
            op: BinOperator::Add,
            right: Box::new(Expr::NumberLiteral(7)),
        },
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            operand: Box::new(Expr::BooleanLiteral(false)),
        },
        Expr::Block(vec![Statement::Regular {
            clauses: vec![],
            is_query: false,
            is_propagate: false,
        }]),
    ];

    for expr in all_exprs {
        drop(expr);
    }
}

#[test]
fn test_deep_expr_drop() {
    let mut expr = Expr::BooleanLiteral(true);
    for _ in 0..1000 {
        expr = Expr::BinOp {
            left: Box::new(expr),
            op: BinOperator::Add,
            right: Box::new(Expr::NumberLiteral(1)),
        };
    }
    drop(expr);
}
#[test]
fn test_expr_debug_formatting() {
    let exprs = vec![
        Expr::StringLiteral("test".to_string()),
        Expr::NumberLiteral(42),
        Expr::BooleanLiteral(true),
        Expr::ArrayLiteral(vec![Expr::NumberLiteral(1)]),
        Expr::IndexAccess {
            array: Box::new(Expr::Word(Word::new("arr"))),
            index: Box::new(Expr::NumberLiteral(0)),
        },
        Expr::Word(Word::new("word")),
        Expr::Phrase(vec![Expr::Word(Word::new("phrase"))]),
        Expr::PropertyAccess {
            owner: Box::new(Expr::Word(Word::new("obj"))),
            property: Box::new(Expr::Word(Word::new("prop"))),
        },
        Expr::Call {
            verb: Word::new("call"),
            arguments: vec![Expr::NumberLiteral(1)],
        },
        Expr::Binding {
            name: Word::new("var"),
            value: Box::new(Expr::NumberLiteral(1)),
        },
        Expr::BinOp {
            left: Box::new(Expr::NumberLiteral(1)),
            op: BinOperator::Add,
            right: Box::new(Expr::NumberLiteral(2)),
        },
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            operand: Box::new(Expr::BooleanLiteral(true)),
        },
        Expr::Block(vec![Statement::Regular {
            clauses: vec![],
            is_query: false,
            is_propagate: false,
        }]),
    ];

    for expr in exprs {
        let s = format!("{:?}", expr);
        assert!(!s.is_empty());
    }
}

#[test]
fn test_statement_debug_formatting() {
    let stmts = vec![
        Statement::Regular {
            clauses: vec![Clause {
                expressions: vec![Expr::NumberLiteral(1)],
            }],
            is_query: true,
            is_propagate: false,
        },
        Statement::TypeDefinition(TypeDef {
            name: Word::new("Type"),
            fields: vec![],
        }),
        Statement::TraitDefinition(TraitDef {
            name: Word::new("Trait"),
            methods: vec![],
        }),
        Statement::TraitImpl(TraitImplDef {
            type_name: Word::new("Type"),
            trait_name: Word::new("Trait"),
            methods: vec![],
        }),
        Statement::TestDeclaration(TestDecl {
            name: "test".to_string(),
            body: vec![],
        }),
    ];

    for stmt in stmts {
        let s = format!("{:?}", stmt);
        assert!(!s.is_empty());
    }
}

#[test]
fn test_program_debug_formatting() {
    let prog = Program {
        statements: vec![Statement::Regular {
            clauses: vec![],
            is_query: false,
            is_propagate: false,
        }],
    };
    let s = format!("{:?}", prog);
    assert!(s.contains("Program"));
}

#[test]
fn test_statement_methods() {
    let stmt = Statement::Regular {
        clauses: vec![],
        is_query: true,
        is_propagate: false,
    };
    assert!(stmt.is_query());
    assert!(!stmt.is_propagate());

    let stmt = Statement::Regular {
        clauses: vec![],
        is_query: false,
        is_propagate: true,
    };
    assert!(!stmt.is_query());
    assert!(stmt.is_propagate());

    let stmt = Statement::TestDeclaration(TestDecl {
        name: "test".to_string(),
        body: vec![],
    });
    assert!(!stmt.is_query());
    assert!(!stmt.is_propagate());
    assert_eq!(stmt.clauses().len(), 0);

    let stmt = Statement::TypeDefinition(TypeDef {
        name: Word::new("t"),
        fields: vec![],
    });
    assert!(!stmt.is_query());
    assert!(!stmt.is_propagate());
    assert_eq!(stmt.clauses().len(), 0);

    let stmt = Statement::TraitDefinition(TraitDef {
        name: Word::new("t"),
        methods: vec![],
    });
    assert!(!stmt.is_query());
    assert!(!stmt.is_propagate());
    assert_eq!(stmt.clauses().len(), 0);

    let stmt = Statement::TraitImpl(TraitImplDef {
        type_name: Word::new("t"),
        trait_name: Word::new("tr"),
        methods: vec![],
    });
    assert!(!stmt.is_query());
    assert!(!stmt.is_propagate());
    assert_eq!(stmt.clauses().len(), 0);
}
