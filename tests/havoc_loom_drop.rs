use glossa::ast::{Expr, Word};
use loom::thread;
use std::sync::Arc;

#[test]
fn test_drop_concurrency() {
    loom::model(|| {
        let string_val = String::from("concurrency test");
        let base_expr = Expr::StringLiteral(string_val);

        let mut tree = base_expr;
        for _ in 0..10 {
            tree = Expr::ArrayLiteral(vec![tree]);
        }

        let expr = Arc::new(tree);

        let expr1 = expr.clone();
        let expr2 = expr.clone();

        let t1 = thread::spawn(move || {
            drop(expr1);
        });

        let t2 = thread::spawn(move || {
            drop(expr2);
        });

        drop(expr);
        t1.join().unwrap();
        t2.join().unwrap();
    });
}
