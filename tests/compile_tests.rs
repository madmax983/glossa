//! Integration tests for the ΓΛΩΣΣΑ compiler pipeline

use glossa::ast::build_ast;
use glossa::semantic::analyze_program;
use glossa::ir::lower_to_hir;
use glossa::codegen::generate_rust;

/// Helper to compile GLOSSA source to Rust code
fn compile(source: &str) -> Result<String, String> {
    let ast = build_ast(source).map_err(|e| e.to_string())?;
    let analyzed = analyze_program(&ast).map_err(|e| e.to_string())?;
    let hir = lower_to_hir(&analyzed);
    Ok(generate_rust(&hir))
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
