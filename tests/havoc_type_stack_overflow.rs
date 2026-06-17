#![allow(missing_docs)]
use glossa::semantic::GlossaType;

/// 👺 Havoc: Stack Overflow in GlossaType
///
/// Warden secured Expr, but GlossaType uses derived Clone, PartialEq, Eq, and Hash.
/// Deep nesting will overflow the stack.
#[test]
#[ignore = "Stack overflow"]
fn havoc_type_stack_overflow() {
    let depth = 50_000;
    let mut t = GlossaType::Number;
    for _ in 0..depth {
        t = GlossaType::List(Box::new(t));
    }

    println!("Cloning deep type (depth {})...", depth);
    let t2 = t.clone();

    println!("Comparing deep type...");
    assert!(t == t2);

    println!("Dropping deep type...");
    drop(t2);
    drop(t);
}
