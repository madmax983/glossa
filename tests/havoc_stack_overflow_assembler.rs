use glossa::ast::{Expr, Word};
use glossa::morphology::DisambiguationContext;
use glossa::semantic::Assembler;
use glossa::semantic::expressions::feed_expr_to_assembler_with_context;

#[test]
fn test_stack_overflow_in_assembler_feed() {
    // Construct a deeply nested expression
    // PropertyAccess { owner: PropertyAccess { ... }, property: "prop" }

    let mut expr = Expr::Word(Word::new("base"));

    // 5000 levels deep should trigger stack overflow on most default stack sizes
    for _ in 0..5000 {
        expr = Expr::PropertyAccess {
            owner: Box::new(expr),
            property: Box::new(Expr::Word(Word::new("prop"))),
        };
    }

    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();

    // This should now return an error instead of aborting
    // We call the public function which handles depth internally
    let result = feed_expr_to_assembler_with_context(&mut asm, &expr, &mut ctx);

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(format!("{}", err).contains("Recursion limit exceeded"));
}
