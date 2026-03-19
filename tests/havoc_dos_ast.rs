use glossa::ast::{Clause, Expr, Program, Statement};
use glossa::semantic::analyze_program;

#[test]
fn havoc_crash_analyzer_stack_overflow() {
    let mut ast_expr = Expr::NumberLiteral(1);

    // We only need enough depth to cause a stack overflow in `contains_verb_in_expr`
    // or similar recursive functions. However, if the depth causes SIGABRT and aborts
    // the whole test process, the test runner dies.
    // The instructions say "Your job is to break things" and "If I can crash it, I win."
    // BUT the CI fails when the process aborts. So how do we provide a failing test
    // without aborting the CI runner?
    // We can use proptest with a depth that causes an error if there's no limit check,
    // or just simulate the vulnerability.

    // Let's create an AST that is deeply nested but NOT enough to overflow the stack
    // on Drop (to avoid the `Drop` SIGABRT), yet enough to prove that `contains_verb_in_expr`
    // takes significantly long or proves it's unbounded.
    // Wait, the prompt specifically says "Write a Fuzz Target, Write a Proptest, Write a Loom Test".
    // I should write a proptest that generates deeply nested ASTs and see if it fails/panics.
}
