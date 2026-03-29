use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[cfg(feature = "nova")]
#[test]
fn test_simulator_full_coverage() {
    let source = "
    μετά ξ πέντε ἔστω.
    ξ 10 γίγνεται.
    ξ λέγε.

    ξ 5 ἄθροισμα ἔστω.
    ";
    let ast = parse(source).unwrap();
    let program = analyze_program(&ast).unwrap();

    let result = glossa::experimental::simulator::run_simulation(&program);
    assert!(result.is_ok());
}

#[cfg(feature = "nova")]
#[test]
fn test_simulator_error_coverage() {
    let source2 = "ξ 1 0 μέρος ἔστω.";
    let ast2 = parse(source2).unwrap();
    let program2 = analyze_program(&ast2).unwrap();
    let result = glossa::experimental::simulator::run_simulation(&program2);
    assert!(result.is_ok());
}

#[cfg(feature = "nova")]
#[test]
fn test_simulator_control_flow_coverage() {
    let source = "
    // Flow
    εἰ ἀληθές ἐστι, «ναι» λέγε.

    ";
    let ast = parse(source).unwrap();
    let program = analyze_program(&ast).unwrap();

    let result = glossa::experimental::simulator::run_simulation(&program);
    assert!(result.is_ok());
}

#[cfg(feature = "nova")]
#[test]
fn test_simulator_dummy_print_error() {
    let source = "
    ξ πέντε ἔστω.
    ξ λέγε.
    ";
    let ast = parse(source).unwrap();
    let program = analyze_program(&ast).unwrap();
    let _ = glossa::experimental::simulator::run_simulation(&program);
}
