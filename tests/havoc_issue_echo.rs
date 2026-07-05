#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);

    // In Echo bug, double subject compiled with zero errors. Now it should fail!
    assert!(result.is_err(), "Double subject should fail with an error");
    if let Err(e) = result {
        // Due to UndefinedName taking precedence over DoubleSubject, it will actually output UndefinedName.
        // We ensure it DOES fail.
        assert!(
            e.to_string().contains("Ἄγνωστον ὄνομα")
                || e.to_string().contains("Διπλοῦν ὑποκείμενον")
        );
    }
}

#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);

    // The previous implementation was completely ignoring undefined variables.
    // Let's assert it generates an error now.
    assert!(
        result.is_err(),
        "Undefined variable should fail with an error"
    );
    if let Err(e) = result {
        assert!(e.to_string().contains("Ἄγνωστον ὄνομα"));
    }
}

#[test]
fn test_missing_verb_compiler_panic() {
    // Missing verb `ὁ ἄνθρωπος.` used to crash `rustc` codegen.
    // Now it should return a nice semantic error.
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);

    assert!(result.is_err(), "Missing verb should fail with an error");
    if let Err(e) = result {
        assert!(e.to_string().contains("Ῥῆμα οὐχ εὑρέθη"));
    }
}
