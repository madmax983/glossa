#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
#[should_panic(expected = "Undefined variable swallowed silently in object position")]
fn test_havoc_undefined_variable_object() {
    let source = "«test» undef τίθησι."; // This uses object fallback
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    if result.is_ok() {
        panic!("Undefined variable swallowed silently in object position");
    }
}

#[test]
#[should_panic(expected = "Undefined variable swallowed silently in subject position")]
fn test_havoc_undefined_variable_subject() {
    let source = "undef «test» λέγει."; // This uses subject fallback
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    if result.is_ok() {
        panic!("Undefined variable swallowed silently in subject position");
    }
}

#[test]
#[should_panic(expected = "Undefined variable swallowed silently in print")]
fn test_havoc_undefined_variable_print() {
    let source = "undef λέγε.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    if result.is_ok() {
        panic!("Undefined variable swallowed silently in print");
    }
}
