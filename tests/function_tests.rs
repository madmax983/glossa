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
    // ξ -> g_x -> g__x, ψ -> g_ps -> g__ps
    assert!(code.contains("g__x") && code.contains("g__ps"));
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
    assert!(code.contains("g__x") || code.contains("*") || code.contains("2"));
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
    // πρόσθεσις -> g__p_r_o_s_th_e_s_i_s
    let name = "g__p_r_o_s_th_e_s_i_s";
    assert!(
        code.contains(name)
            && (code.contains(&format!("{}(", name)) || code.contains(&format!("{} (", name)))
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
    // διπλασιασμος -> g__d_i_p_l_a_s_i_a_s_m_o_s
    let name = "g__d_i_p_l_a_s_i_a_s_m_o_s";
    assert!(
        code.matches(name).count() >= 3,
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
    // τοπικον -> g__t_o_p_i_k_o_n
    assert!(code.contains("let g__t_o_p_i_k_o_n"));
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
