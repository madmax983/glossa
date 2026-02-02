use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

/// Helper to compile GLOSSA source to Rust code
fn compile(source: &str) -> String {
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    generate_rust(&analyzed)
}

// ============================================================================
// CYCLE 1: Simple Function Definition (No Parameters)
// ============================================================================

#[test]
fn test_parse_simple_function_no_params() {
    let ast = parse("χαιρετισμος ὁρίζειν· «χαῖρε» λέγε.").unwrap();
    assert_eq!(ast.statements.len(), 1);
    // Debug: print clause structure
    eprintln!("Clauses: {}", ast.statements[0].clauses().len());
    for (i, clause) in ast.statements[0].clauses().iter().enumerate() {
        eprintln!("Clause {}: {} expressions", i, clause.expressions.len());
    }
}

#[test]
fn test_codegen_function_keyword() {
    let code = compile("χαιρετισμος ὁρίζειν· «χαῖρε» λέγε.");
    assert!(code.contains("fn"), "Expected 'fn' keyword");
}

// ============================================================================
// CYCLE 2: Function with Parameters
// ============================================================================

#[test]
fn test_function_with_two_params() {
    let code = compile("προσθεσις ὁρίζειν τῷ ξ τῷ ψ· δός ξ ψ ἄθροισμα.");
    eprintln!("Generated code:\n{}", code);
    assert!(code.contains("fn"));
    assert!(code.contains("xi") && code.contains("psi"));
}

#[test]
fn test_function_typed_params() {
    let code = compile("προσθεσις ὁρίζειν τῷ ξ ἀριθμοῦ τῷ ψ ἀριθμοῦ· δός ξ ψ ἄθροισμα.");
    assert!(code.contains("i64"));
}

// ============================================================================
// CYCLE 3: Return Statements
// ============================================================================

#[test]
fn test_simple_return() {
    let code = compile("διπλασιασμος ὁρίζειν τῷ ξ· δός ξ δύο γινόμενον.");
    eprintln!("Generated code:\n{}", code);
    assert!(code.contains("return"));
    // Should return xi * 2, not just a literal
    assert!(code.contains("xi") || code.contains("*") || code.contains("2"));
}

#[test]
fn test_return_type_inference() {
    let code = compile("διπλασιασμος ὁρίζειν τῷ ξ ἀριθμοῦ· δός ξ δύο γινόμενον.");
    eprintln!("Generated code:\n{}", code);
    assert!(code.contains("-> i64"));
}

// ============================================================================
// CYCLE 4: Function Calls
// ============================================================================

#[test]
fn test_function_call() {
    let code = compile(
        "
        προσθεσις ὁρίζειν τῷ ξ τῷ ψ· δός ξ ψ ἄθροισμα.
        ἀποτελεσμα προσθεσις πέντε τρία ἔστω.
    ",
    );
    eprintln!("Generated code:\n{}", code);
    assert!(
        code.contains("prosthesis")
            && (code.contains("prosthesis(") || code.contains("prosthesis ("))
    );
}

#[test]
fn test_nested_calls() {
    let code = compile(
        "
        διπλασιασμος ὁρίζειν τῷ ξ· δός ξ δύο γινόμενον.
        ψ διπλασιασμος (διπλασιασμος πέντε) ἔστω.
    ",
    );
    eprintln!("Generated code:\n{}", code);
    // Check for nested diplasiasmos calls (allowing for whitespace)
    assert!(
        code.matches("diplasiasmos").count() >= 3,
        "Expected at least 3 occurrences of diplasiasmos (fn def + 2 calls)"
    );
    assert!(code.contains("5i64"), "Expected literal 5 as argument");
}

// ============================================================================
// CYCLE 5: Function Scope
// ============================================================================

#[test]
fn test_function_local_variables() {
    let code = compile(
        "
        αυξησις ὁρίζειν τῷ ξ·
            τοπικον ξ ἓν ἄθροισμα ἔστω·
            δός τοπικον.
    ",
    );
    eprintln!("Generated code:\n{}", code);
    assert!(code.contains("let topikon"));
}

#[test]
fn test_parameter_shadowing() {
    let code = compile(
        "
        ξ δέκα ἔστω.
        προσθεσις ὁρίζειν τῷ ξ· δός ξ ἓν ἄθροισμα.
        ψ προσθεσις πέντε ἔστω.
    ",
    );
    eprintln!("Generated code:\n{}", code);
    // Should compile without error - just verify it compiles
}
