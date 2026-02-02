use glossa::ast::{Expr, Word, Statement, Clause};
use glossa::semantic::analyze_program;

#[test]
fn test_stack_overflow_recursion() {
    // 1. Define 'φ' (phi) using parser
    let func_def_src = "φ ὁρίζειν τῷ χ · δός χ.";
    let mut ast = glossa::parser::parse(func_def_src).expect("Failed to parse function definition");

    // 2. Manually build deep AST to bypass parser recursion limits
    // φ(φ(φ(...))) -> Phrase([Word("φ"), Phrase([Word("φ"), ...])])

    let depth = 500;
    let mut expr = Expr::NumberLiteral(1);

    for _ in 0..depth {
        expr = Expr::Phrase(vec![
            Expr::Word(Word::new("φ")),
            expr
        ]);
    }

    // Wrap expression in a statement that triggers function call analysis
    // Structure: res φ (nested...) ἔστω.
    // Assembler:
    // - res (Nom) -> Subject
    // - φ (Nom) -> Extra Nominative (Function Name)
    // - (nested) -> Nested Phrase
    // - ἔστω (Verb) -> Binding Verb

    let call_stmt = Statement::Regular {
        clauses: vec![Clause { expressions: vec![
            Expr::Word(Word::new("res")),
            Expr::Word(Word::new("φ")),
            expr,
            Expr::Word(Word::new("ἔστω"))
        ] }],
        is_query: false,
        is_propagate: false,
    };

    // Add to program
    ast.statements.push(call_stmt);

    println!("Analyzing deep expression (depth {})...", depth);
    let result = analyze_program(&ast);

    // 3. Assert we got an error (recursion limit exceeded) instead of crashing
    assert!(result.is_err(), "Expected recursion limit error, but got success");

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Recursion limit"), "Error message should mention recursion limit. Got: {}", err_msg);
}
