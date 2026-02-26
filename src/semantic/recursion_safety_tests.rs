
use crate::ast::{Expr, Word};
use crate::semantic::expressions::{MAX_RECURSION_DEPTH, feed_expr_to_assembler_with_context};
use crate::semantic::{Assembler, DisambiguationContext};

// Helper to create a nested structure of a specific type
fn make_nested_expr(depth: usize, constructor: impl Fn(Expr) -> Expr) -> Expr {
    let mut expr = Expr::Word(Word::new("x"));
    for _ in 0..depth {
        expr = constructor(expr);
    }
    // Wrap in PropertyAccess to trigger the check_cloning_depth_safety
    Expr::PropertyAccess {
        owner: Box::new(expr),
        property: Box::new(Expr::Word(Word::new("len"))),
    }
}

#[test]
fn test_check_safety_phrase_recursion() {
    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();
    let depth = MAX_RECURSION_DEPTH + 10;

    let expr = make_nested_expr(depth, |e| Expr::Phrase(vec![e]));

    let result = feed_expr_to_assembler_with_context(&mut asm, &expr, &mut ctx);
    assert!(result.is_err(), "Should catch recursion in Phrase");
}

#[test]
fn test_check_safety_array_recursion() {
    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();
    let depth = MAX_RECURSION_DEPTH + 10;

    let expr = make_nested_expr(depth, |e| Expr::ArrayLiteral(vec![e]));

    let result = feed_expr_to_assembler_with_context(&mut asm, &expr, &mut ctx);
    assert!(result.is_err(), "Should catch recursion in ArrayLiteral");
}

#[test]
fn test_check_safety_index_access_recursion_array() {
    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();
    let depth = MAX_RECURSION_DEPTH + 10;

    let expr = make_nested_expr(depth, |e| Expr::IndexAccess {
        array: Box::new(e),
        index: Box::new(Expr::NumberLiteral(0)),
    });

    let result = feed_expr_to_assembler_with_context(&mut asm, &expr, &mut ctx);
    assert!(result.is_err(), "Should catch recursion in IndexAccess (array)");
}

#[test]
fn test_check_safety_index_access_recursion_index() {
    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();
    let depth = MAX_RECURSION_DEPTH + 10;

    let expr = make_nested_expr(depth, |e| Expr::IndexAccess {
        array: Box::new(Expr::ArrayLiteral(vec![])),
        index: Box::new(e),
    });

    let result = feed_expr_to_assembler_with_context(&mut asm, &expr, &mut ctx);
    assert!(result.is_err(), "Should catch recursion in IndexAccess (index)");
}

#[test]
fn test_check_safety_binop_recursion_left() {
    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();
    let depth = MAX_RECURSION_DEPTH + 10;

    let expr = make_nested_expr(depth, |e| Expr::BinOp {
        left: Box::new(e),
        op: crate::ast::BinOperator::Add,
        right: Box::new(Expr::NumberLiteral(1)),
    });

    let result = feed_expr_to_assembler_with_context(&mut asm, &expr, &mut ctx);
    assert!(result.is_err(), "Should catch recursion in BinOp (left)");
}

#[test]
fn test_check_safety_binop_recursion_right() {
    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();
    let depth = MAX_RECURSION_DEPTH + 10;

    let expr = make_nested_expr(depth, |e| Expr::BinOp {
        left: Box::new(Expr::NumberLiteral(1)),
        op: crate::ast::BinOperator::Add,
        right: Box::new(e),
    });

    let result = feed_expr_to_assembler_with_context(&mut asm, &expr, &mut ctx);
    assert!(result.is_err(), "Should catch recursion in BinOp (right)");
}

#[test]
fn test_check_safety_unary_op_recursion() {
    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();
    let depth = MAX_RECURSION_DEPTH + 10;

    let expr = make_nested_expr(depth, |e| Expr::UnaryOp {
        op: crate::ast::UnaryOperator::Not,
        operand: Box::new(e),
    });

    let result = feed_expr_to_assembler_with_context(&mut asm, &expr, &mut ctx);
    assert!(result.is_err(), "Should catch recursion in UnaryOp");
}

#[test]
fn test_check_safety_binding_recursion() {
    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();
    let depth = MAX_RECURSION_DEPTH + 10;

    let expr = make_nested_expr(depth, |e| Expr::Binding {
        name: Word::new("x"),
        value: Box::new(e),
    });

    let result = feed_expr_to_assembler_with_context(&mut asm, &expr, &mut ctx);
    assert!(result.is_err(), "Should catch recursion in Binding");
}

#[test]
fn test_check_safety_call_recursion() {
    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();
    let depth = MAX_RECURSION_DEPTH + 10;

    let expr = make_nested_expr(depth, |e| Expr::Call {
        verb: Word::new("f"),
        arguments: vec![e],
    });

    let result = feed_expr_to_assembler_with_context(&mut asm, &expr, &mut ctx);
    assert!(result.is_err(), "Should catch recursion in Call");
}
