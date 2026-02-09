use glossa::ast::{Expr, Program, Statement, Word};
use glossa::semantic::analyze_program;

#[test]
fn test_property_access_stack_overflow() {
    // Manually build deep AST with PropertyAccess
    // Depth needs to be sufficient to cause stack overflow.
    // On many systems 5000 is enough for debug builds, 20000+ for release.
    let depth = 5000;
    let mut expr = Expr::Word(Word::new("root"));

    for _ in 0..depth {
        expr = Expr::PropertyAccess {
            owner: Box::new(expr),
            property: Box::new(Expr::Word(Word::new("prop"))),
        };
    }

    // Wrap in a simple statement: "expr;"
    let stmt = Statement::Regular {
        clauses: vec![vec![expr]],
        is_query: false,
        is_propagate: false,
    };

    let program = Program {
        statements: vec![stmt],
    };

    println!("Analyzing deep property access (depth {})...", depth);
    // This should panic with stack overflow if not protected
    let result = analyze_program(&program);

    // We expect this to FAIL with a proper error message if fixed,
    // or CRASH if not fixed.
    match result {
        Ok(_) => panic!("Should have failed with recursion limit error"),
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("Recursion limit"),
                "Expected recursion limit error, got: {}",
                msg
            );
        }
    }
}
