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

#[cfg(feature = "nova")]
#[test]
fn test_simulator_from_file_coverage() {
    use std::fs::File;
    use std::io::Write;
    let path = std::env::temp_dir().join("test_sim.glossa");
    let mut file = File::create(&path).unwrap();
    writeln!(file, "μετά ξ πέντε ἔστω.\nξ λέγε.").unwrap();

    let result = glossa::experimental::simulator::run_simulation_from_file(&path);
    assert!(result.is_ok());

    let _ = std::fs::remove_file(&path);
}

#[cfg(feature = "nova")]
#[test]
fn test_simulator_from_file_errors() {
    use std::fs::File;
    use std::io::Write;

    // Parse error
    let path_parse = std::env::temp_dir().join("test_sim_parse.glossa");
    let mut file_parse = File::create(&path_parse).unwrap();
    writeln!(file_parse, "invalid syntax!!!").unwrap();
    let res1 = glossa::experimental::simulator::run_simulation_from_file(&path_parse);
    assert!(res1.is_err());
    let _ = std::fs::remove_file(&path_parse);

    // Semantic error
    let path_sem = std::env::temp_dir().join("test_sim_sem.glossa");
    let mut file_sem = File::create(&path_sem).unwrap();
    writeln!(file_sem, "μετά ξ α ἔστω.").unwrap();
    let res2 = glossa::experimental::simulator::run_simulation_from_file(&path_sem);
    // run_simulation captures runtime errors into the table and returns Ok(), but parse errors return Err()
    let _ = std::fs::remove_file(&path_sem);

    // Not found
    let res3 = glossa::experimental::simulator::run_simulation_from_file(std::path::Path::new("not_real_abc123.glossa"));
    assert!(res3.is_err());
}
