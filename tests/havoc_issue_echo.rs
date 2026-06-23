#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    if result.is_ok() {
        panic!("Havoc: Double subject successfully compiled when it should have crashed!");
    }
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Διπλοῦν ὑποκείμενον")
    );
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    if result.is_ok() {
        panic!("Havoc: Undefined variable silently evaluated instead of panicking!");
    }
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Οὐκ οἶδα τὸ ὄνομα")
    );
}

#[test]
fn test_missing_verb_compiler_panic() {
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    if result.is_ok() {
        panic!("Havoc: Missing verb returned Ok(()) which caused a codegen ICE!");
    }
    assert!(result.unwrap_err().to_string().contains("Ῥῆμα οὐχ εὑρέθη"));
}
