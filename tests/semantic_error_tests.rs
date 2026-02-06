use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_undefined_variable_error() {
    let source = "άγνωστος λέγε.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("is not defined") || err.contains("οὐχ ὡρίσθη"));
}

#[test]
fn test_undefined_variable_object_error() {
    let source = "ξ πέντε ἔστω. ξ άγνωστος λέγε."; // "Say x unknown" -> unknown is object
    // Wait, "ξ άγνωστος λέγε" might be parsed as "x unknown say".
    // If "άγνωστος" is parsed as object.
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("is not defined") || err.contains("οὐχ ὡρίσθη"));
}
