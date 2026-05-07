#![allow(missing_docs)]
use glossa::codegen::generate_statement_code;
use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType};
use std::thread;

#[test]
#[ignore]
fn test_codegen_stack_overflow() {
    let child = thread::Builder::new()
        .stack_size(1024 * 1024 * 2)
        .spawn(|| {
            let depth = 40000;
            // Construct a deeply nested AnalyzedExpr structure.
            // Bypasses the parser's recursion limit, going straight to the core semantic AST.
            let mut expr = AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            };
            for _ in 0..depth {
                expr = AnalyzedExpr {
                    expr: AnalyzedExprKind::PropertyAccess {
                        owner: Box::new(expr),
                        property: "prop".into(),
                    },
                    glossa_type: GlossaType::Unknown,
                };
            }

            let stmt = AnalyzedStatement::Expression(vec![expr]);

            println!("Starting codegen...");
            // This crashes either during codegen TokenStream processing or implicit drop afterwards,
            // proving that the semantic AST lacks `stacker::maybe_grow` protections.
            let _code = generate_statement_code(&stmt);
            println!("Finished codegen!");

            // Note: If you reach here, it will definitely crash on the implicit Drop.
        })
        .unwrap();
    child.join().unwrap();
}
