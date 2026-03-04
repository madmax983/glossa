use glossa::codegen::generate_rust;
use glossa::morphology::lexicon::UnaryOp;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};

#[test]
fn test_warden_neg_overflow_defense() {
    // This test verifies that negating a number generates a safe `.checked_neg()`
    // call instead of a simple `-`, preventing integer overflows (e.g. negating i64::MIN).
    let number_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(-9223372036854775808), // i64::MIN
        glossa_type: GlossaType::Number,
    };

    let neg_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::UnaryOp {
            op: UnaryOp::Neg,
            operand: Box::new(number_expr),
        },
        glossa_type: GlossaType::Number,
    };

    let stmt = AnalyzedStatement::Expression(vec![neg_expr]);

    let program = AnalyzedProgram {
        statements: vec![stmt],
        scope: Scope::new(),
    };

    let code = generate_rust(&program);

    println!("Generated code:\n{}", code);

    // Verify the fix: we should see `.checked_neg().expect(...)`
    assert!(code.contains("checked_neg"));
    assert!(code.contains("expect"));
    assert!(code.contains("arithmetic overflow"));

    // Verify we are not generating the old unsafe `-` logic for numbers
    // `quote! { -#operand_tokens }`
    // It should look something like: `(-9223372036854775808).checked_neg().expect("arithmetic overflow")`
    assert!(!code.replace(" ", "").contains("-(x)"));
}
