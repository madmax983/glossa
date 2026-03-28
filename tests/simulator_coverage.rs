use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[cfg(feature = "nova")]
#[test]
fn test_simulator_full_coverage() {
    let source = "
    μετά ξ πέντε ἔστω.
    ξ 10 γίγνεται.
    ξ λέγε.

    // Some expression to hit the expression branch
    ξ 5 ἄθροισμα ἔστω.
    ";
    let ast = parse(source).unwrap();
    let program = analyze_program(&ast).unwrap();

    let result = glossa::experimental::simulator::run_simulation(&program);
    assert!(result.is_ok());
}
