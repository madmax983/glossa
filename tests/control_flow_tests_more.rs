use glossa::{codegen::generate_rust, parser::parse, semantic::analyze_program};

fn compile_to_rust(source: &str) -> String {
    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    generate_rust(&analyzed)
}

#[test]
fn test_match_single_word_variable_missing() {
    let source = "κατὰ ἄγνωστο· ἄγνωστο ᾖ, «ἄγνωστο» λέγε· ἄλλο ᾖ, «ἄλλο» λέγε.";
    let output = compile_to_rust(source);
    assert!(output.contains("match"));
}

#[test]
fn test_skip_first_word_and_parse_single_literal_number() {
    let source = "εἰ 1, 1 λέγε.";
    let output = compile_to_rust(source);
    assert!(output.contains("if"));
}

#[test]
fn test_skip_first_word_and_parse_single_literal_numeral_word() {
    let source = "εἰ πέντε, πέντε λέγε.";
    let output = compile_to_rust(source);
    assert!(output.contains("if"));
}

#[test]
fn test_missing_verb_bare_subject() {
    let source = "ὁ ἄνθρωπος.";
    let ast = glossa::parser::parse(source).expect("Failed to parse");
    let result = glossa::semantic::analyze_program(&ast);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Ῥῆμα οὐχ εὑρέθη"));
}

#[test]
fn test_missing_verb_bare_subject_object() {
    let source = "ὁ ἄνθρωπος τὸν ἄνθρωπον.";
    let ast = glossa::parser::parse(source).expect("Failed to parse");
    let result = glossa::semantic::analyze_program(&ast);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Ῥῆμα οὐχ εὑρέθη"));
}
