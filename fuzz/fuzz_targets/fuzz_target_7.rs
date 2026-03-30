#![no_main]

use libfuzzer_sys::fuzz_target;
use glossa::ast::{Expr, Statement, Clause, Word};
use glossa::semantic::analyzer::analyze_program;
use glossa::ast::Program;

fuzz_target!(|data: &[u8]| {
    // Generate a deeply nested AST to test stack overflow protection
    let depth = 50000;
    if data.len() < 10 { return; }

    let mut stmt = Statement::Regular {
        clauses: vec![Clause {
            expressions: vec![Expr::Word(Word::new("test"))],
        }],
        is_query: false,
        is_propagate: false,
    };

    for _ in 0..depth {
        stmt = Statement::Regular {
            clauses: vec![Clause {
                expressions: vec![Expr::Block(vec![stmt])],
            }],
            is_query: false,
            is_propagate: false,
        };
    }

    let program = Program {
        statements: vec![stmt],
    };

    let _ = analyze_program(&program);
});
