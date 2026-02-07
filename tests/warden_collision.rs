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
    // "π" (pi) -> "p" -> "g_p" (using a known variable from lexicon to ensure parsing)
    // Internal index variable is "idx".

    let source = "π [1] ἔστω. π[0] λέγε.";
    let ast = parse(source).expect("Parse failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    let code = generate_rust(&analyzed);

    // User variable prefixed
    assert!(code.contains("let mut g_p"));

    // Internal variable for indexing (not prefixed by g_, but by logic in codegen)
    assert!(code.contains("let idx = 0"));

    // Usage matches
    assert!(code.contains("g_p"));
    assert!(code.contains("[idx as usize]"));
}
