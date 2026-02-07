use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_identifier_prefixing() {
    // Verify that user identifiers are prefixed with 'g_' to avoid collisions.
    // "ξ" (xi) -> "x" -> "g_x"
    let source = "ξ πέντε ἔστω.";
    let ast = parse(source).expect("Parse failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    let code = generate_rust(&analyzed);

    assert!(code.contains("let g_x = 5"));
    assert!(!code.contains("let x = 5"));
}

#[test]
fn test_collision_avoidance_with_internal_vars() {
    // Verify that user variables don't collide with internal generated variables (e.g. 'idx').
    // Internal variable `idx` is generated during IndexAccess.
    // We use [1 2] (multiple elements) to avoid ambiguity with IndexAccess [1].
    // "ξ [1 2] ἔστω. ξ[0] λέγε."
    // ξ -> g_x

    let source = "ξ [1 2] ἔστω. ξ[0] λέγε.";
    let ast = parse(source).expect("Parse failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    let code = generate_rust(&analyzed);

    // User variable prefixed
    assert!(code.contains("let mut g_x"));

    // Internal variable for indexing
    assert!(code.contains("let idx ="));

    // Usage matches
    assert!(code.contains("g_x"));
    // Ensure `idx` is used for indexing
    assert!(code.contains("[idx as usize]"));
}
