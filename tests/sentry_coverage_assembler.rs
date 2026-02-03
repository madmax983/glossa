use glossa::ast::{BinOperator, Clause, Expr, Statement, UnaryOperator, Word};
use glossa::morphology::DisambiguationContext;
use glossa::semantic::Assembler;
use glossa::semantic::expressions::feed_expr_to_assembler_with_context;

#[test]
fn test_assembler_coverage_variants() {
    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();

    // 1. Phrase (non-nested)
    // "1 2"
    let phrase_expr = Expr::Phrase(vec![Expr::NumberLiteral(1), Expr::NumberLiteral(2)]);
    let res = feed_expr_to_assembler_with_context(&mut asm, &phrase_expr, &mut ctx);
    assert!(res.is_ok());

    // 2. Phrase (nested)
    // "(1)" - nested phrase handling
    let nested_phrase = Expr::Phrase(vec![Expr::Phrase(vec![Expr::NumberLiteral(1)])]);
    let res = feed_expr_to_assembler_with_context(&mut asm, &nested_phrase, &mut ctx);
    assert!(res.is_ok());

    // 3. Call
    // "add 1"
    let call_expr = Expr::Call {
        verb: Word::new("λεγε"), // Use a known verb so it analyzes
        arguments: vec![Expr::NumberLiteral(1)],
    };
    let res = feed_expr_to_assembler_with_context(&mut asm, &call_expr, &mut ctx);
    assert!(res.is_ok());

    // 4. Binding
    // "x = 1" (Binding expr, not statement)
    let binding_expr = Expr::Binding {
        name: Word::new("χ"),
        value: Box::new(Expr::NumberLiteral(1)),
    };
    let res = feed_expr_to_assembler_with_context(&mut asm, &binding_expr, &mut ctx);
    assert!(res.is_ok());

    // 5. BinOp
    // "1 + 2"
    let binop_expr = Expr::BinOp {
        left: Box::new(Expr::NumberLiteral(1)),
        op: BinOperator::Add,
        right: Box::new(Expr::NumberLiteral(2)),
    };
    let res = feed_expr_to_assembler_with_context(&mut asm, &binop_expr, &mut ctx);
    assert!(res.is_ok());

    // 6. UnaryOp (Not Unwrap)
    // "!x"
    let unary_expr = Expr::UnaryOp {
        op: UnaryOperator::Not,
        operand: Box::new(Expr::BooleanLiteral(true)),
    };
    let res = feed_expr_to_assembler_with_context(&mut asm, &unary_expr, &mut ctx);
    assert!(res.is_ok());

    // 7. Block
    let block_expr = Expr::Block(vec![Statement::Regular {
        clauses: vec![Clause {
            expressions: vec![Expr::NumberLiteral(1)],
        }],
        is_query: false,
        is_propagate: false,
    }]);
    let res = feed_expr_to_assembler_with_context(&mut asm, &block_expr, &mut ctx);
    assert!(res.is_ok());

    // 8. ArrayLiteral
    let array_expr = Expr::ArrayLiteral(vec![Expr::NumberLiteral(1)]);
    let res = feed_expr_to_assembler_with_context(&mut asm, &array_expr, &mut ctx);
    assert!(res.is_ok());

    // 9. IndexAccess
    let index_expr = Expr::IndexAccess {
        array: Box::new(Expr::ArrayLiteral(vec![])),
        index: Box::new(Expr::NumberLiteral(0)),
    };
    let res = feed_expr_to_assembler_with_context(&mut asm, &index_expr, &mut ctx);
    assert!(res.is_ok());
}
