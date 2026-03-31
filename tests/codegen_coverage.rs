use glossa::semantic::AnalyzedStatement;
use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};

#[test]
fn test_generate_statement_expression_optimization() {
    let exprs = vec![AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(42),
        glossa_type: GlossaType::Number,
    }];
    let stmt = AnalyzedStatement::Expression(exprs);
    let code = glossa::codegen::generate_statement_code(&stmt);
    assert!(code.contains("42i64"));
}

#[test]
fn test_generate_print_optimization() {
    let args = vec![AnalyzedExpr {
        expr: AnalyzedExprKind::StringLiteral("hello".to_string()),
        glossa_type: GlossaType::String,
    }];
    let stmt = AnalyzedStatement::Print(args);
    let code = glossa::codegen::generate_statement_code(&stmt);
    assert!(code.contains("println ! (\"{}\" , \"hello\")"));
}

#[test]
fn test_generate_collection_array_optimization() {
    // Array literals are generated via expressions wrapped in statements
    let elems = vec![AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(10),
        glossa_type: GlossaType::Number,
    }];
    let expr = AnalyzedExprKind::ArrayLiteral(elems);
    let stmt = AnalyzedStatement::Expression(vec![AnalyzedExpr {
        expr,
        glossa_type: GlossaType::Unknown,
    }]);
    let code = glossa::codegen::generate_statement_code(&stmt);
    assert!(code.contains("vec ! [10i64]"));
}

#[test]
fn test_generate_method_call_optimization() {
    let receiver = Box::new(AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(5),
        glossa_type: GlossaType::Number,
    });
    let args = vec![AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(10),
        glossa_type: GlossaType::Number,
    }];
    let expr = AnalyzedExprKind::MethodCall {
        receiver,
        method: "abs".to_string().into(),
        args,
    };
    let stmt = AnalyzedStatement::Expression(vec![AnalyzedExpr {
        expr,
        glossa_type: GlossaType::Number,
    }]);
    let code = glossa::codegen::generate_statement_code(&stmt);
    assert!(code.contains("g_abs (10i64)"));
}

#[test]
fn test_generate_trait_method_call_optimization() {
    let receiver = Box::new(AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(5),
        glossa_type: GlossaType::Number,
    });
    let args = vec![AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(10),
        glossa_type: GlossaType::Number,
    }];
    let expr = AnalyzedExprKind::MethodCall {
        receiver,
        method: "add".to_string().into(),
        args,
    };
    let stmt = AnalyzedStatement::Expression(vec![AnalyzedExpr {
        expr,
        glossa_type: GlossaType::Number,
    }]);
    let code = glossa::codegen::generate_statement_code(&stmt);
    assert!(code.contains("g_add (10i64)"));
}

#[test]
fn test_generate_function_call_optimization() {
    let args = vec![AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(10),
        glossa_type: GlossaType::Number,
    }];
    let expr = AnalyzedExprKind::FunctionCall {
        func: "add".to_string().into(),
        args,
    };
    let stmt = AnalyzedStatement::Expression(vec![AnalyzedExpr {
        expr,
        glossa_type: GlossaType::Number,
    }]);
    let code = glossa::codegen::generate_statement_code(&stmt);
    assert!(code.contains("g_add (10i64)"));
}

#[test]
fn test_generate_closure_optimization() {
    let expr = AnalyzedExprKind::Lambda {
        params: vec!["x".to_string().into()],
        body: Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(10),
            glossa_type: GlossaType::Number,
        }),
        capture_mode: glossa::semantic::CaptureMode::Borrow,
    };
    let stmt = AnalyzedStatement::Expression(vec![AnalyzedExpr {
        expr,
        glossa_type: GlossaType::Unknown,
    }]);
    let code = glossa::codegen::generate_statement_code(&stmt);
    assert!(code.contains("| g_x | 10i64"));
}

#[test]
fn test_generate_unary_op_optimization() {
    let stmt = AnalyzedStatement::Expression(vec![AnalyzedExpr {
        expr: AnalyzedExprKind::UnaryOp {
            op: glossa::morphology::lexicon::UnaryOp::Neg,
            operand: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(10),
                glossa_type: GlossaType::Number,
            }),
        },
        glossa_type: GlossaType::Number,
    }]);
    let code = glossa::codegen::generate_statement_code(&stmt);
    assert!(code.contains("checked_neg ()"));
}
