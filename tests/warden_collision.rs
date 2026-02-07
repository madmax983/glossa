use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_variable_collision_with_keyword() {
    // "struct" is a keyword in Rust.
    // We use "ο struct" (the struct) to force it into the Subject slot.
    // "ο" (article) sets Nominative context, so "struct" (unknown word) is resolved as Nominative.
    let source = "ο struct 5 ἔστω.";
    let ast = parse(source).expect("Parse failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    let code = generate_rust(&analyzed);

    // With sanitization prefix, it produces "let g_struct = 5;" which is valid Rust.
    assert!(code.contains("let g_struct = 5"));
}

#[test]
fn test_variable_collision_with_internal_var() {
    // "idx" is used internally in IndexAccess generation.
    // "ο idx" forces Subject.
    let source = "ο idx [1] ἔστω. idx[0] λέγε.";
    let ast = parse(source).expect("Parse failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    let code = generate_rust(&analyzed);

    // It should avoid the collision by prefixing user variable
    assert!(code.contains("let idx = 0;")); // Internal var for index access logic
    // g_idx might be mutable because it's an array
    assert!(code.contains("let mut g_idx = vec![1]"));

    // The usage should refer to g_idx
    assert!(code.contains("g_idx"));
    assert!(code.contains("[idx as usize]"));
}
