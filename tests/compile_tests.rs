//! Integration tests for the ΓΛΩΣΣΑ compiler pipeline

use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

/// Helper to compile GLOSSA source to Rust code
fn compile(source: &str) -> Result<String, String> {
    let ast = parse(source).map_err(|e| e.to_string())?;
    let analyzed = analyze_program(&ast).map_err(|e| e.to_string())?;
    Ok(generate_rust(&analyzed))
}

#[test]
fn test_hello_cosmos() {
    let code = compile("«χαῖρε κόσμε» λέγε.").unwrap();
    assert!(code.contains("println"));
    assert!(code.contains("χαῖρε κόσμε"));
}

#[test]
fn test_variable_binding() {
    let code = compile("ξ πέντε ἔστω.").unwrap();
    assert!(code.contains("let xi"));
    assert!(code.contains("5"));
}

#[test]
fn test_variable_binding_and_use() {
    let code = compile("ξ πέντε ἔστω. ξ λέγε.").unwrap();
    assert!(code.contains("let xi = 5"));
    assert!(code.contains("println"));
    assert!(code.contains("xi"));
}

#[test]
fn test_number_literal() {
    let code = compile("42 λέγε.").unwrap();
    assert!(code.contains("42"));
}

#[test]
fn test_multiple_statements() {
    let code = compile("α πέντε ἔστω. β δέκα ἔστω. α λέγε.").unwrap();
    assert!(code.contains("let alpha = 5"));
    assert!(code.contains("let beta = 10"));
}

#[test]
fn test_greek_numeral_word() {
    let code = compile("ξ δέκα ἔστω.").unwrap();
    assert!(code.contains("10"));
}

#[test]
fn test_preserves_greek_in_strings() {
    let code = compile("«Ἑλλάς» λέγε.").unwrap();
    assert!(code.contains("Ἑλλάς"));
}

#[test]
fn test_mutable_binding() {
    let code = compile("μετά ξ πέντε ἔστω.").unwrap();
    assert!(code.contains("let mut xi"));
    assert!(code.contains("5"));
}

#[test]
fn test_assignment_codegen() {
    let code = compile("μετά ξ πέντε ἔστω. ξ δέκα γίγνεται.").unwrap();
    assert!(code.contains("let mut xi = 5"));
    assert!(code.contains("xi = 10"));
}

#[test]
fn test_immutable_assignment_error() {
    let result = compile("ξ πέντε ἔστω. ξ δέκα γίγνεται.");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("ἀμετάβλητόν"));
}

#[test]
fn test_undefined_assignment_error() {
    let result = compile("ξ δέκα γίγνεται.");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("οὐχ ὡρίσθη"));
}

#[test]
fn test_mutable_binding_and_reassignment() {
    let code = compile("μετά ξ πέντε ἔστω. ξ λέγε. ξ δέκα γίγνεται. ξ λέγε.").unwrap();
    assert!(code.contains("let mut xi = 5"));
    assert!(code.contains("xi = 10"));
    assert!(code.matches("println").count() >= 2);
}

#[test]
fn test_assignment_missing_value_error() {
    let result = compile("μετά ξ πέντε ἔστω. ξ γίγνεται.");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("δεῖ τιμῆς"));
}
