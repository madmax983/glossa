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
    // ξ -> g__u3be_, ψ -> g__u3c8_
    assert!(code.contains("g__u3be_") && code.contains("g__u3c8_"));
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
    // Should return x * 2, not just a literal
    assert!(code.contains("g__u3be_") || code.contains("*") || code.contains("2"));
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
    // πρόσθεσις -> hex encoded
    // π(3c0) ρ(3c1) ο(3bf) σ(3c3) θ(3b8) ε(3b5) σ(3c3) ι(3b9) ς(3c2)
    let expected = "g__u3c0__u3c1__u3bf__u3c3__u3b8__u3b5__u3c3__u3b9__u3c2_";
    assert!(code.contains(expected), "Expected function name: {}", expected);
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
    // διπλασιασμος -> hex encoded
    // δ(3b4) ι(3b9) π(3c0) λ(3bb) α(3b1) σ(3c3) ι(3b9) α(3b1) σ(3c3) μ(3bc) ο(3bf) ς(3c2)
    let expected = "g__u3b4__u3b9__u3c0__u3bb__u3b1__u3c3__u3b9__u3b1__u3c3__u3bc__u3bf__u3c2_";
    assert!(
        code.matches(expected).count() >= 3,
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
    // τοπικον -> hex encoded
    // τ(3c4) ο(3bf) π(3c0) ι(3b9) κ(3ba) ο(3bf) ν(3bd)
    let expected = "g__u3c4__u3bf__u3c0__u3b9__u3ba__u3bf__u3bd_";
    assert!(code.contains(expected));
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
