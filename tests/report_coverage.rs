use glossa::report::{CompilationReport, GlossaReport, ProgramStats};
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};
use std::time::Duration;

// Helper to create a dummy expression (boxed)
fn dummy_expr_box() -> Box<AnalyzedExpr> {
    Box::new(dummy_expr())
}

// Helper to create a dummy expression
fn dummy_expr() -> AnalyzedExpr {
    AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(42),
        glossa_type: GlossaType::Number,
    }
}

// Helper to create a dummy binding statement
fn dummy_binding(name: &str) -> AnalyzedStatement {
    AnalyzedStatement::Binding {
        name: name.into(),
        value: dummy_expr(),
        mutable: false,
    }
}

#[test]
fn test_program_stats_visitor_coverage() {
    // Construct a program that hits every statement type in ProgramStats::visit_statement
    let mut statements = Vec::new();

    // 1. Binding
    statements.push(dummy_binding("x"));

    // 2. Assignment
    statements.push(AnalyzedStatement::Assignment {
        name: "x".into(),
        value: dummy_expr(),
    });

    // 3. Print
    statements.push(AnalyzedStatement::Print(vec![dummy_expr()]));

    // 4. Expression
    statements.push(AnalyzedStatement::Expression(vec![dummy_expr()]));

    // 5. Query
    statements.push(AnalyzedStatement::Query(vec![dummy_expr()]));

    // 6. If (with Else)
    statements.push(AnalyzedStatement::If {
        condition: dummy_expr_box(),
        then_body: vec![dummy_binding("y")],
        else_body: Some(vec![dummy_binding("z")]),
    });

    // 7. While
    statements.push(AnalyzedStatement::While {
        condition: dummy_expr_box(),
        body: vec![dummy_binding("w")],
    });

    // 8. For
    statements.push(AnalyzedStatement::For {
        variable: "i".into(),
        iterator: dummy_expr_box(),
        body: vec![dummy_binding("j")],
    });

    // 9. Match
    statements.push(AnalyzedStatement::Match {
        scrutinee: dummy_expr_box(),
        arms: vec![(dummy_expr(), vec![dummy_binding("k")])],
    });

    // 10. FunctionDef (in body)
    statements.push(AnalyzedStatement::FunctionDef {
        name: "inner".into(),
        params: vec![],
        body: vec![dummy_binding("l")],
        return_type: None,
    });

    // 11. TestDeclaration
    statements.push(AnalyzedStatement::TestDeclaration {
        name: "test".into(),
        body: vec![dummy_binding("m")],
    });

    // 12. Return
    statements.push(AnalyzedStatement::Return {
        value: Some(dummy_expr_box()),
    });

    // 13. Break/Continue (no-op visitor but good for completeness)
    statements.push(AnalyzedStatement::Break);
    statements.push(AnalyzedStatement::Continue);

    let program = AnalyzedProgram {
        statements,
        scope: Scope::default(),
    };

    let stats = ProgramStats::new(&program);

    // Verify counts to ensure visitor ran
    assert!(stats.statement_count > 10);
    assert!(stats.binding_count > 0);
    assert!(stats.conditional_count > 0);
    assert!(stats.loop_count > 0);
    assert!(stats.expression_count > 0);
    // Depth should be at least 1 (nested statements)
    assert!(stats.max_depth >= 1);
}

#[test]
fn test_program_stats_expr_visitor_coverage() {
    // Construct expressions covering all AnalyzedExprKind variants visited
    let mut exprs = Vec::new();

    // PropertyAccess
    exprs.push(AnalyzedExpr {
        expr: AnalyzedExprKind::PropertyAccess {
            owner: dummy_expr_box(),
            property: "prop".into(),
        },
        glossa_type: GlossaType::Number,
    });

    // VerbCall
    exprs.push(AnalyzedExpr {
        expr: AnalyzedExprKind::VerbCall {
            verb: "verb".into(),
            args: vec![dummy_expr()],
        },
        glossa_type: GlossaType::Unit,
    });

    // BinOp
    exprs.push(AnalyzedExpr {
        expr: AnalyzedExprKind::BinOp {
            op: glossa::morphology::lexicon::BinaryOp::Add,
            left: dummy_expr_box(),
            right: dummy_expr_box(),
        },
        glossa_type: GlossaType::Number,
    });

    // UnaryOp
    exprs.push(AnalyzedExpr {
        expr: AnalyzedExprKind::UnaryOp {
            op: glossa::morphology::lexicon::UnaryOp::Neg,
            operand: dummy_expr_box(),
        },
        glossa_type: GlossaType::Number,
    });

    // Range
    exprs.push(AnalyzedExpr {
        expr: AnalyzedExprKind::Range {
            start: dummy_expr_box(),
            end: dummy_expr_box(),
            inclusive: true,
        },
        glossa_type: GlossaType::Unknown,
    });

    // ArrayLiteral
    exprs.push(AnalyzedExpr {
        expr: AnalyzedExprKind::ArrayLiteral(vec![dummy_expr()]),
        glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
    });

    // Option/Result variants
    exprs.push(AnalyzedExpr {
        expr: AnalyzedExprKind::Some(dummy_expr_box()),
        glossa_type: GlossaType::Option(Box::new(GlossaType::Number)),
    });

    // IndexAccess
    exprs.push(AnalyzedExpr {
        expr: AnalyzedExprKind::IndexAccess {
            array: dummy_expr_box(),
            index: dummy_expr_box(),
        },
        glossa_type: GlossaType::Number,
    });

    // FunctionCall
    exprs.push(AnalyzedExpr {
        expr: AnalyzedExprKind::FunctionCall {
            func: "func".into(),
            args: vec![dummy_expr()],
        },
        glossa_type: GlossaType::Unit,
    });

    // MethodCall
    exprs.push(AnalyzedExpr {
        expr: AnalyzedExprKind::MethodCall {
            receiver: dummy_expr_box(),
            method: "method".into(),
            args: vec![dummy_expr()],
        },
        glossa_type: GlossaType::Unit,
    });

    // Lambda
    exprs.push(AnalyzedExpr {
        expr: AnalyzedExprKind::Lambda {
            params: vec![],
            body: dummy_expr_box(),
            capture_mode: glossa::semantic::CaptureMode::Borrow,
        },
        glossa_type: GlossaType::Unknown,
    });

    // Assert
    exprs.push(AnalyzedExpr {
        expr: AnalyzedExprKind::Assert {
            condition: dummy_expr_box(),
        },
        glossa_type: GlossaType::Unit,
    });

    // Wrap in a statement so stats visitor hits them
    let stmt = AnalyzedStatement::Expression(exprs);
    let program = AnalyzedProgram {
        statements: vec![stmt],
        scope: Scope::default(),
    };

    let stats = ProgramStats::new(&program);
    assert!(stats.expression_count >= 10);
}

#[test]
fn test_glossa_report_display_full_coverage() {
    // Construct a program that produces all stats for full report coverage
    let mut statements = Vec::new();

    // Add loops to trigger loop count
    for _ in 0..4 {
        statements.push(AnalyzedStatement::While {
            condition: dummy_expr_box(),
            // Nest a statement inside to increase depth to 1
            body: vec![AnalyzedStatement::Expression(vec![])],
        });
    }

    // Create a scope with dummy functions and types
    let mut scope = Scope::default();
    scope.define_function("test_func", vec![GlossaType::Number], Some(GlossaType::String));
    scope.define_function("func2", vec![], None);
    scope.define_function("func3", vec![], None);

    scope.define_type("Type1", GlossaType::Number);
    scope.define_type("Type2", GlossaType::String);

    let program = AnalyzedProgram {
        statements,
        scope,
    };

    let report = GlossaReport::new(&program, "test.gl".into());

    let output = format!("{}", report);

    // Verify all headers and values are present
    assert!(output.contains("ΑΝΑΦΟΡΑ ΓΛΩΣΣΗΣ"));
    assert!(output.contains("Συναρτήσεις"));
    assert!(output.contains("Τύποι"));
    assert!(output.contains("Βρόχοι"));
    // Max depth should be at least 1 from the while loop
    // But visit_statement starts at depth 0.
    // While loop body is empty in this test case: body: vec![].
    // So recursive call happens on body statements at depth + 1.
    // If body is empty, recursion stops.
    // However, stats.visit_statement is called on the While loop itself.
    // Let's check max_depth logic: self.max_depth = self.max_depth.max(depth);
    // Initial call is depth 0.
    // So if we don't nest, max_depth is 0.
    // And display logic: if self.stats.max_depth > 0 { ... }
    // So we need nesting to trigger > 0.

    // Verify function table
    assert!(output.contains("ΣΥΝΑΡΤΗΣΕΙΣ"));
    assert!(output.contains("test_func"));
    assert!(output.contains("Ἀριθμός")); // Param type
    assert!(output.contains("Ὄνομα")); // Return type
}

#[test]
fn test_compilation_report_display_coverage() {
    let stats = ProgramStats {
        statement_count: 100,
        function_count: 5,
        ..Default::default()
    };

    let report = CompilationReport {
        input_path: "/src/main.gl".into(),
        output_path: "/bin/main".into(),
        stats,
        duration: Duration::from_millis(500),
    };

    let output = format!("{}", report);

    assert!(output.contains("ΣΥΜΠΕΡΑΣΜΑ"));
    assert!(output.contains("/src/main.gl"));
    assert!(output.contains("/bin/main"));
    assert!(output.contains("100")); // statements
    assert!(output.contains("500.00ms"));
    assert!(output.contains("ΕΠΙΤΥΧΙΑ"));
}
