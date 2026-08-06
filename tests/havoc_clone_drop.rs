//! Havoc tests for ensuring deep AST cloning and dropping don't overflow the stack.
//!
//! This module contains tests that build deeply nested Abstract Syntax Trees
//! and verify that `clone` and `drop` operations are stack-safe (using `stacker`).

use glossa::ast::{Expr, Word};
use std::thread;

#[test]
fn test_clone_stack_overflow() {
    let child = thread::Builder::new()
        .stack_size(1024 * 1024 * 10) // generous but still droppable
        .spawn(|| {
            let depth = 50000;
            let mut expr = Expr::Word(Word::new("root"));
            for _ in 0..depth {
                expr = Expr::PropertyAccess {
                    owner: Box::new(expr),
                    property: Box::new(Expr::Word(Word::new("prop"))),
                };
            }

            // Stacker should protect Clone according to ast.rs
            let expr2 = expr.clone();
            // Also test dropping it
            drop(expr2);
            drop(expr);
        })
        .unwrap();
    child.join().unwrap();
}
